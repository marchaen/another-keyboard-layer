mod debugger;
mod events;
mod send_input;
mod translation;

use std::{
    collections::HashMap, os::windows::io::AsRawHandle, ptr, sync::mpsc, thread, time::Duration,
};

use windows::Win32::{
    Foundation::{BOOL, HANDLE, HMODULE, LPARAM, LRESULT, WPARAM},
    System::{
        Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT},
        Threading::GetThreadId,
    },
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        HHOOK, HOOKPROC, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WINDOWS_HOOK_ID, WM_APP, WM_KEYDOWN,
        WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use debugger::Debugger;
use events::{Action, Event, EventProcessor, Key, KeyCombination};
use send_input::send_key_combination;
use translation::{translate_to_character, windows_to_virtual_key, VirtualKey};

fn main() {
    Debugger::init();

    let mut mappings = HashMap::new();

    mappings.insert(key_combination!("h"), key_combination!("LeftArrow"));
    mappings.insert(key_combination!("j"), key_combination!("DownArrow"));
    mappings.insert(key_combination!("k"), key_combination!("UpArrow"));
    mappings.insert(key_combination!("l"), key_combination!("RightArrow"));

    // Some mor testing
    mappings.insert(key_combination!("t"), key_combination!("😊"));
    mappings.insert(
        key_combination!("LShift" + "t"),
        key_combination!("A" + "B" + "C"),
    );
    mappings.insert(key_combination!("LAlt" + "t"), key_combination!("."));
    mappings.insert(key_combination!("LMeta" + "t"), key_combination!("^" + "a"));

    Debugger::write(&format!("Mappings: {mappings:?}"));

    unsafe {
        STATE.event_processor.replace(EventProcessor::new(
            Key::Virtual(VirtualKey::CapsLock),
            Some(
                [Key::Virtual(VirtualKey::Escape)]
                    .as_slice()
                    .try_into()
                    .unwrap(),
            ),
            mappings,
        ))
    };

    let (keyboard_hook_sender, keyboard_hook_receiver) = mpsc::channel();

    let message_queue = thread::spawn(move || {
        // Important: The hook has to be registered from the same thread in
        // which the message queue is running. That's why there is a need
        // to explicitly send the handle to the main thread.
        let _ = keyboard_hook_sender.send(HookHandle::register(
            "global raw keyboard".to_owned(),
            WH_KEYBOARD_LL,
            Some(raw_keyboard_input_hook),
        ));
        drop(keyboard_hook_sender);

        run_message_queue();
    });

    let thread_id = {
        // How to convert std library handle to windows-rs handle: https://stackoverflow.com/a/73574560
        let thread_handle = HANDLE(message_queue.as_raw_handle() as isize);

        // See https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadid
        unsafe { GetThreadId(thread_handle) }
    };

    let keyboard_hook = keyboard_hook_receiver
        .recv()
        .expect("Should receive hook from message queue thread almost immediately.");
    drop(keyboard_hook_receiver);

    // TODO: Is the following comment still relevant?

    // Explicit shutdown callback is needed because windows will kill the
    // process without terminating the message queue (which would have been done
    // by sending a WM_QUIT message) if the console window is closed or ctrl + c
    // is pressed.
    //
    // Everything related to the shutdown hook in this prototype will not be
    // needed in the real library implementation because the c# clients will
    // take care of graceful shutdown instead.
    set_shutdown_callback(move || {
        Debugger::write(&format!("Shutdown {thread_id}"));
        // See post thread message https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew
        // See WM_QUIT message https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-quit
        let result = unsafe { PostThreadMessageW(thread_id, WM_APP + 1, WPARAM(0), LPARAM(0)) };
        Debugger::write(&format!("Shutdown result {result:?}"));

        drop(keyboard_hook);
        Debugger::destroy();
    });

    // In the real library the client will continue the execution of the program
    // at this point.

    loop {
        thread::sleep(Duration::from_secs(10));
        Debugger::write("Heartbeat");
    }
}

static mut CALLBACK: Option<Box<dyn FnOnce()>> = None;

fn set_shutdown_callback(callback: impl FnOnce() + 'static) -> bool {
    let previous_callback = unsafe { CALLBACK.replace(Box::new(callback)) };

    if previous_callback.is_some() {
        return true;
    }

    unsafe { SetConsoleCtrlHandler(Some(raw_shutdown_handler), true) }.as_bool()
}

unsafe extern "system" fn raw_shutdown_handler(ctrltype: u32) -> BOOL {
    match ctrltype {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
            if let Some(callback) = unsafe { CALLBACK.take() } {
                Debugger::write("Calling shutdown hook.");
                callback();
            }
        }
        _ => (),
    }

    true.into()
}

struct HookHandle {
    hook: HHOOK,
    name: String,
}

impl HookHandle {
    fn register(name: String, id: WINDOWS_HOOK_ID, listener: HOOKPROC) -> Self {
        Debugger::write(&format!("Register {name} listener hook."));

        let register_result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(id, listener, HMODULE(0), 0)
        };

        match register_result {
            Ok(hook) => {
                Debugger::write(&format!(
                    "Successfully registered the {name} listener hook ({hook:?})."
                ));
                Self { hook, name }
            }
            Err(error) => panic!(
                "Trying to register a {} listener failed: {} ({})",
                name,
                error.message().to_string_lossy(),
                error.code()
            ),
        }
    }

    fn unregister(&self) {
        Debugger::write(&format!("Unregister global {} listener hook", self.name));

        let result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
            UnhookWindowsHookEx(self.hook)
        };

        Debugger::write(&format!(
            "Unregister global {} listener result: {result:?}",
            self.name
        ));
    }
}

impl Drop for HookHandle {
    fn drop(&mut self) {
        self.unregister();
    }
}

pub struct NativeKeyboardInputHook {
    event_processor: Option<EventProcessor>,
    currently_writing: bool,
}

impl NativeKeyboardInputHook {
    const fn new() -> Self {
        Self {
            event_processor: None,
            currently_writing: false,
        }
    }
}

// TODO: Write safety comments for this state (about calling this hook from the
// same thread from the message queue (run_message_queue) which is single threaded)
static mut STATE: NativeKeyboardInputHook = NativeKeyboardInputHook::new();

// See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
unsafe extern "system" fn raw_keyboard_input_hook(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let event_processor = STATE.event_processor.as_mut();
    let currently_writing = &mut STATE.currently_writing;

    // As documented we can't handle any events that have a code lower than zero.
    // We should instead pass them to the next hook and return their result.
    if code < 0 || *currently_writing || event_processor.is_none() {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }

    let event_processor = event_processor.unwrap();

    let event_pointer: *const KBDLLHOOKSTRUCT = std::mem::transmute(lparam);
    let event = event_pointer.as_ref().unwrap();

    // Try to translate the character from the keyboard event or use the unicode
    // replacement character "�" (https://compart.com/en/unicode/U+FFFD).
    let mut key = Key::Text(translate_to_character(event).unwrap_or('\u{FFFD}'));

    if let Some(virtual_key) = windows_to_virtual_key(event.vkCode as u16) {
        key = Key::Virtual(virtual_key);
    }

    let action = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN=> Action::Press,
        WM_KEYUP | WM_SYSKEYUP => Action::Release,
        _ => unreachable!("See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc#wparam-in"),
    };

    let event = Event { action, key };
    let change_request = event_processor.process(event);

    Debugger::write(&format!("{event:?} => {change_request:?}"));

    match change_request {
        events::ChangeEventRequest::Block => LRESULT(1),
        events::ChangeEventRequest::ReplaceWith(key_combination) => {
            *currently_writing = true;
            send_key_combination(key_combination);
            *currently_writing = false;
            LRESULT(1)
        }
        events::ChangeEventRequest::None => CallNextHookEx(HHOOK(0), code, wparam, lparam),
    }
}

// A low level keyboard hook needs a message queue to be running in the case of
// this application that means the GetMessage-Function will block indefinitely.
//
// That also means we could make rewrite the loop to stop after receiving one
// message so that there is a way to terminate the message queue from another
// thread is needed in the actual akl-core-system-lib.
//
// See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc#remarks
fn run_message_queue() {
    Debugger::write("Running message queue and block until end");

    let mut message = MSG::default();

    // We don't need to handle the return type we only want to listen to the
    // first message that gets posted anyway
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage
    let result = unsafe { GetMessageW(ptr::addr_of_mut!(message), None, 0, 0) };
    Debugger::write(&format!(
        "Got message shuting down message queue {1:#X} (Status: {0})",
        result.0, message.message
    ));
}
