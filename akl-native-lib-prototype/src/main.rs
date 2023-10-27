mod debugger;
mod translation;

use std::ptr;

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HMODULE, LPARAM, LRESULT, WPARAM},
        System::{
            Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT},
            LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryW},
        },
        UI::WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
            UnhookWindowsHookEx, HHOOK, HOOKPROC, KBDLLHOOKSTRUCT, MSG, WH_GETMESSAGE,
            WH_KEYBOARD_LL, WINDOWS_HOOK_ID, WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST, WM_KEYUP,
            WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use debugger::Debugger;
use translation::{translate_to_character, windows_to_virtual_key};

fn main() {
    Debugger::init();

    let keyboard_hook = HookHandle::register(
        "global raw keyboard".to_owned(),
        WH_KEYBOARD_LL,
        Some(raw_keyboard_input_hook),
        None,
    );

    let character_listener = ListenerHandle::load(
        "akl_translated_character_hook",
        "raw_translated_character_hook",
    );

    let character_hook = HookHandle::register(
        "translated character".to_owned(),
        WH_GETMESSAGE,
        character_listener.listener,
        character_listener.dll,
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
        drop(character_hook);
        drop(character_listener);
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

// TODO: Research if there is a way to guarantee that the listener won't be
// called after the handle is dropped. (Maybe something with lifetimes and
// phantom data could be used here.)
struct ListenerHandle {
    dll_name: String,
    listener_name: String,
    dll: Option<HMODULE>,
    listener: HOOKPROC,
}

impl ListenerHandle {
    fn load<S: Into<String>>(dll_name: S, listener_name: S) -> Self {
        let dll_name = dll_name.into();
        let listener_name = listener_name.into();

        Debugger::write(&format!(
            "Load dll {dll_name} for listener hook {listener_name}"
        ));

        assert!(
            listener_name.is_ascii(),
            "The name of the listener \"{listener_name}\" must be ascii only."
        );

        let dll_name_utf16 = dll_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw
        // and https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types
        // for information about invariants. The type which is needed here
        // (pcwstr) is called LPCWSTR in the windows documentation.
        let load_result = unsafe { LoadLibraryW(PCWSTR::from_raw(dll_name_utf16.as_ptr())) };

        match load_result {
            Ok(handle) => {
                Debugger::write(&format!("Successfully loaded dll {dll_name}",));

                // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress
                // for details. The type which is needed here is called lpcstr
                // and has to be ascii only.
                let maybe_listener =
                    unsafe { GetProcAddress(handle, PCSTR::from_raw(listener_name.as_ptr())) };

                // Safety: We control both this program and the target dll which
                // means the specified method will never not be a hook function.
                let listener: HOOKPROC = unsafe { std::mem::transmute(maybe_listener) };

                assert!(
                    listener.is_some(),
                    "Couldn't locate function {listener_name} in dll {dll_name}"
                );

                Self {
                    dll_name,
                    listener_name,
                    dll: Some(handle),
                    listener,
                }
            }
            Err(error) => panic!(
                "Trying to load dll {dll_name} failed: {} ({})",
                error.message().to_string_lossy(),
                error.code()
            ),
        }
    }

    fn unload(&mut self) {
        if self.dll.is_none() {
            return;
        }

        self.listener = None;
        let dll = self.dll.take().unwrap();

        Debugger::write(&format!(
            "Unload dll {} and listener hook {}",
            self.dll_name, self.listener_name
        ));

        // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary
        let result = unsafe { FreeLibrary(dll) };

        Debugger::write(&format!(
            "Unload dll {} and listener hook {} result: {result:?}",
            self.dll_name, self.listener_name
        ));
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.unload();
    }
}

struct HookHandle {
    hook: HHOOK,
    name: String,
}

impl HookHandle {
    fn register(
        name: String,
        id: WINDOWS_HOOK_ID,
        listener: HOOKPROC,
        dll: Option<HMODULE>,
    ) -> Self {
        Debugger::write(&format!("Register {name} listener hook."));

        let register_result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(id, listener, dll.unwrap_or(HMODULE(0)), 0)
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

    match wparam.0 as u32 {
        WM_KEYDOWN => Debugger::write(&format!("{formatted_event} Down")),
        WM_KEYUP => Debugger::write(&format!("{formatted_event} Up")),
        WM_SYSKEYDOWN => Debugger::write(&format!("{formatted_event} SysDown")),
        WM_SYSKEYUP => Debugger::write(&format!("{formatted_event} SysUp")),
        _ => (),
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

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
