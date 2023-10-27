mod translation;

use std::{io::Write, net::TcpStream, ptr};

use translation::{translate_to_character, windows_to_virtual_key};

use windows::Win32::{
    Foundation::{GetLastError, HMODULE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

static mut DEBUGGER: Option<Box<TcpStream>> = None;

fn init_debugger(server: TcpStream) {
    unsafe { DEBUGGER = Some(Box::new(server)) };
}

fn debug(line: &str) {
    let mut line = line.to_owned();
    line.push('\n');

    unsafe {
        DEBUGGER
            .as_mut()
            .expect("Connection to debug server should have been established.")
            .write(line.as_bytes())
            .unwrap()
    };
}

fn main() {
    let debug_server = TcpStream::connect("127.0.0.1:7777")
        .expect("Debug server should run for prototyping the windows hook.");

    println!(
        "Connecting to debug server from address \"{}\".",
        debug_server
            .local_addr()
            .expect("Should be able to get local client address.")
    );

    init_debugger(debug_server);

    let _hook = KeyboardHookHandle::register();
    run_message_queue();
}

// TODO: Make sure that the handle / hook is unregistered in the real
// applications (cli und gui), dafür kann ich dann in c# folgendes verwenden:
// https://learn.microsoft.com/en-us/dotnet/api/system.appdomain.processexit?view=net-7.0&redirectedfrom=MSDN
struct KeyboardHookHandle(HHOOK);

impl KeyboardHookHandle {
    fn register() -> Self {
        debug("Register hook");

        let register_result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(raw_keyboard_input_hook), HMODULE(0), 0)
        };

        match register_result {
            Ok(hook) => {
                debug(&format!("Register result: {hook:?}"));
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
        debug("Unregister hook");

        let result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
            UnhookWindowsHookEx(self.0)
        };

        debug(&format!("Unregister result: {result:?}"));
    }
}

impl Drop for KeyboardHookHandle {
    fn drop(&mut self) {
        self.unregister_hook();
    }
}

fn run_message_queue() {
    let mut message = MSG::default();

    debug("Running message queue");
    loop {
        // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage
        let result =
            unsafe { GetMessageW(ptr::addr_of_mut!(message), None, WM_KEYFIRST, WM_KEYLAST) };

        debug(&format!("Message result: {}", result.0));

        // Zero means exit, -1 is an error and anything else indicates that the
        // message should be dispatched.
        match result.0 {
            0 => break,
            -1 => {
                let error_message = unsafe { GetLastError() }
                    .to_hresult()
                    .message()
                    .to_string_lossy();

                debug(&format!("Error retrieving message: {error_message}"));
            }
            _ => {
                debug("Dispatching message");

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
        WM_KEYDOWN => debug(&format!("{formatted_event} Down")),
        WM_KEYUP => debug(&format!("{formatted_event} Up")),
        WM_SYSKEYDOWN => debug(&format!("{formatted_event} SysDown")),
        WM_SYSKEYUP => debug(&format!("{formatted_event} SysUp")),
        _ => unreachable!("See values for wparam here: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc")
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}
