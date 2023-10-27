use std::{io::Write, net::TcpStream, ptr};

use windows::Win32::{
    Foundation::{GetLastError, HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardState, ToUnicodeEx},
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
            HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST,
            WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

// TODO: Make sure that the handle / hook is unregistered in the real
// applications (cli und gui), dafür kann ich dann in c# folgendes verwenden:
// https://learn.microsoft.com/en-us/dotnet/api/system.appdomain.processexit?view=net-7.0&redirectedfrom=MSDN
struct Handle(HHOOK);

impl Drop for Handle {
    fn drop(&mut self) {
        unregister_hook(self.0);
    }
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

    let _hook = Handle(register_hook());
    run_message_queue();
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

// See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
unsafe extern "system" fn native_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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

    let formatted_event = format!(
        "Time: {} Raw: {} Virtual: {:#X} Character: '{translation}'",
        event.time, event.scanCode, event.vkCode
    );

    match wparam.0 as u32 {
        WM_KEYDOWN => debug(&format!("Down {formatted_event}")),
        WM_KEYUP => debug(&format!("Up {formatted_event}")),
        WM_SYSKEYDOWN => debug(&format!("SysDown {formatted_event}")),
        WM_SYSKEYUP => debug(&format!("SysUp {formatted_event}")),
        _ => unreachable!("See values for wparam here: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc")
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

fn translate_to_character(event: &KBDLLHOOKSTRUCT) -> Option<char> {
    let mut keyboard_state = [0u8; 256];

    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardstate
    let state_retrieval_result = unsafe { GetKeyboardState(&mut keyboard_state) };

    // Getting the state failed and thus translating the event is impossible.
    if state_retrieval_result.0 == 0 {
        return None;
    }

    // TODO: Cache the layout in a global variable and listen to
    // WM_INPUTLANGCHANGE events to update it instead of querying it every time.
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardlayout
    let keyboard_layout = unsafe { GetKeyboardLayout(0) };

    // The documentation doesn't specify how many characters can be returned max
    // for one key event. When more than one characters translated successfully
    // the first one will be returned.
    let mut raw_character = [0u16; 8];

    // Setting the third bit of the flags value makes sure to keep the keyboard
    // state as is. Not doing this breaks the dead characters (^, `, ´).
    let flags = 0b100;

    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex
    let translation_result = unsafe {
        ToUnicodeEx(
            event.vkCode,
            event.scanCode,
            &keyboard_state,
            &mut raw_character,
            flags,
            keyboard_layout,
        )
    };

    if translation_result == 0 {
        return None;
    }

    let char_count = translation_result.unsigned_abs() as usize;
    char::decode_utf16(raw_character[..char_count].iter().copied()).find_map(Result::ok)
}

fn register_hook() -> HHOOK {
    debug("Register hook");

    let register_result = unsafe {
        // Details and safety see:
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(native_hook), HMODULE(0), 0)
    };

    match register_result {
        Ok(hook) => {
            debug(&format!("Register result: {hook:?}"));
            hook
        }
        Err(error) => panic!(
            "Trying to register a global keyboard event listener failed: {} ({})",
            error.message().to_string_lossy(),
            error.code()
        ),
    }
}

fn unregister_hook(hook: HHOOK) {
    debug("Unregister hook");

    let result = unsafe {
        // Details and safety see:
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
        UnhookWindowsHookEx(hook)
    };

    debug(&format!("Unregister result: {result:?}"));
}
