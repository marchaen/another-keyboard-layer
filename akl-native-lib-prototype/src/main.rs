mod debugger;
mod translation;

use std::ptr;

use windows::Win32::{
    Foundation::{GetLastError, HMODULE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use debugger::Debugger;
use translation::{translate_to_character, windows_to_virtual_key};

fn main() {
    Debugger::init();

    let _keyboard_hook = KeyboardHookHandle::register();
    run_message_queue();

    Debugger::destroy();
}

// TODO: Make sure that the handle / hook is unregistered in the real
// applications (cli und gui), following can be used to do that
// https://learn.microsoft.com/en-us/dotnet/api/system.appdomain.processexit?view=net-7.0&redirectedfrom=MSDN
struct KeyboardHookHandle(HHOOK);

impl KeyboardHookHandle {
    fn register() -> Self {
        Debugger::write("Register hook");

        let register_result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(raw_keyboard_input_hook), HMODULE(0), 0)
        };

        match register_result {
            Ok(hook) => {
                Debugger::write(&format!("Register result: {hook:?}"));
                Self(hook)
            }
            Err(error) => panic!(
                "Trying to register a global keyboard event listener failed: {} ({})",
                error.message().to_string_lossy(),
                error.code()
            ),
        }
    }

    fn unregister_hook(&mut self) {
        Debugger::write("Unregister hook");

        let result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
            UnhookWindowsHookEx(self.0)
        };

        Debugger::write(&format!("Unregister result: {result:?}"));
    }
}

impl Drop for KeyboardHookHandle {
    fn drop(&mut self) {
        self.unregister_hook();
    }
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

    let formatted_event = {
        if let Some(virtual_key) = windows_to_virtual_key(event.vkCode as u16) {
            format!(
                "Time: {} Raw: {:0>3} Key: {virtual_key:?} ({:#X})",
                event.time, event.scanCode, event.vkCode
            )
        } else {
            format!(
                "Time: {} Raw: {:0>3} Key: {translation}",
                event.time, event.scanCode
            )
        }
    };

    match wparam.0 as u32 {
        WM_KEYDOWN => Debugger::write(&format!("{formatted_event} Down")),
        WM_KEYUP => Debugger::write(&format!("{formatted_event} Up")),
        WM_SYSKEYDOWN => Debugger::write(&format!("{formatted_event} SysDown")),
        WM_SYSKEYUP => Debugger::write(&format!("{formatted_event} SysUp")),
        _ => unreachable!("See values for wparam here: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc")
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}
