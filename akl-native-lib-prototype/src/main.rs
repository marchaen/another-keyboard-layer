mod debugger;
mod send_input;
mod translation;

use std::ptr;

use windows::Win32::{
    Foundation::{GetLastError, BOOL, HMODULE, LPARAM, LRESULT, WPARAM},
    System::Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HHOOK, HOOKPROC, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
        WINDOWS_HOOK_ID, WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use debugger::Debugger;
use send_input::{type_char_key, type_virtual_key};
use translation::{translate_to_character, windows_to_virtual_key, VirtualKey};

fn main() {
    Debugger::init();

    let keyboard_hook = HookHandle::register(
        "global raw keyboard".to_owned(),
        WH_KEYBOARD_LL,
        Some(raw_keyboard_input_hook),
    );

    // Explicit shutdown callback is needed because windows will kill the
    // process without terminating the message queue (which would have been done
    // by sending a WM_QUIT message) if the console window is closed or ctrl + c
    // is pressed.
    //
    // Everything related to the shutdown hook in this prototype will not be
    // needed in the real library implementation because the c# clients will
    // take care of graceful shutdown instead.
    set_shutdown_callback(move || {
        drop(keyboard_hook);
        Debugger::destroy();
    });

    run_message_queue();
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

static mut BLOCKED: bool = false;

// See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
unsafe extern "system" fn raw_keyboard_input_hook(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // As documented we can't handle any events that have a code lower than zero.
    // We should instead pass them to the next hook and return their result.
    if code < 0 {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }

    let event_pointer: *const KBDLLHOOKSTRUCT = std::mem::transmute(lparam);
    let event = event_pointer.as_ref().unwrap();

    // Try to translate the character from the keyboard event or use the unicode
    // replacement character "�" (https://compart.com/en/unicode/U+FFFD).
    let translation = translate_to_character(event).unwrap_or('\u{FFFD}');

    let formatted_event = windows_to_virtual_key(event.vkCode as u16).map_or_else(
        || {
            format!(
                "Time: {} Raw: {:0>3} Key: {translation}",
                event.time, event.scanCode
            )
        },
        |virtual_key| {
            format!(
                "Time: {} Raw: {:0>3} Key: {virtual_key:?} ({:#X})",
                event.time, event.scanCode, event.vkCode
            )
        },
    );

    let virtual_key = windows_to_virtual_key(event.vkCode as u16);

    match wparam.0 as u32 {
        WM_KEYDOWN => {
            Debugger::write(&format!("{formatted_event} Down"));

            // TODO: In the core system lib there should be an extra variable
            // for if we are writing a character ourselves. So that there aren't
            // any weird bugs because of processing events that were caused by
            // a call to send input from in here.

            if BLOCKED {
                BLOCKED = false;

                match translation {
                    'h' => {
                        type_virtual_key(VirtualKey::LeftArrow);

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    'j' => {
                        type_virtual_key(VirtualKey::DownArrow);

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    'k' => {
                        type_virtual_key(VirtualKey::UpArrow);

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    'l' => {
                        type_virtual_key(VirtualKey::RightArrow);

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    't' => {
                        type_char_key('H');
                        type_char_key('a');
                        type_char_key('l');
                        type_char_key('l');
                        type_char_key('o');
                        type_char_key(',');
                        type_char_key(' ');
                        type_char_key('W');
                        type_char_key('e');
                        type_char_key('l');
                        type_char_key('t');
                        type_char_key('!');

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    's' => {
                        // Testing that characters outside the base plain work
                        // too, because they are going to be encoded in two
                        // u16s and send as separate key strokes.
                        //
                        // See https://en.wikipedia.org/wiki/Plane_(Unicode)
                        // and https://stackoverflow.com/a/22308727
                        type_char_key('😀');

                        BLOCKED = true;
                        return LRESULT(1);
                    }
                    _ => (),
                }
            }

            if let Some(virtual_key) = virtual_key {
                if virtual_key == VirtualKey::CapsLock {
                    Debugger::write("Start blocking...");
                    BLOCKED = true;
                    return LRESULT(1);
                }
            }
        }
        WM_KEYUP => {
            Debugger::write(&format!("{formatted_event} Up"));

            if let Some(virtual_key) = virtual_key {
                if virtual_key == VirtualKey::CapsLock {
                    BLOCKED = false;
                    Debugger::write("Stopped blocking.");
                    type_virtual_key(VirtualKey::Escape);
                    return LRESULT(1);
                }
            }
        }
        WM_SYSKEYDOWN => Debugger::write(&format!("{formatted_event} SysDown")),
        WM_SYSKEYUP => Debugger::write(&format!("{formatted_event} SysUp")),
        _ => (),
    }

    if BLOCKED {
        return LRESULT(1);
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
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
    let mut message = MSG::default();

    Debugger::write("Running message queue");
    loop {
        // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage
        let result =
            unsafe { GetMessageW(ptr::addr_of_mut!(message), None, WM_KEYFIRST, WM_KEYLAST) };

        Debugger::write(&format!("Message result: {}", result.0));

        // Zero means exit, -1 is an error and anything else indicates that the
        // message should be dispatched.
        match result.0 {
            0 => break,
            -1 => {
                let error_message = unsafe { GetLastError() }
                    .to_hresult()
                    .message()
                    .to_string_lossy();

                Debugger::write(&format!("Error retrieving message: {error_message}"));
            }
            _ => {
                Debugger::write("Translate message");
                // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage
                // Returns if the message was translated (WM_CHAR event) or not.
                unsafe { TranslateMessage(ptr::addr_of!(message)) };

                Debugger::write("Dispatching message");
                // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-dispatchmessage
                // Note: The return value should be ignored.
                unsafe { DispatchMessageW(ptr::addr_of!(message)) };
            }
        }
    }
}
