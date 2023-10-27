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

macro_rules! windows_virtual_key_code_to_virtual_key_translations {
    ($($name: ident = $translation: expr),*,) => {
            fn windows_to_virtual_key(windows_key: u16) -> Option<VirtualKey> {
                match windows_key {
                    $($translation => Some(VirtualKey::$name),)*
                    _ => None,
                }
            }
    };
}

// See https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
windows_virtual_key_code_to_virtual_key_translations!(
    Back = 0x08,
    Tab = 0x09,
    Clear = 0x0c,
    Return = 0x0d,
    Shift = 0x10,
    Control = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    KanaOrHangul = 0x15,
    ImeOn = 0x16,
    Junja = 0x17,
    Final = 0x18,
    KanjiOrHanja = 0x19,
    ImeOff = 0x1a,
    Escape = 0x1b,
    ImeConvert = 0x1c,
    ImeNonconvert = 0x1d,
    ImeAccept = 0x1e,
    ImeModechange = 0x1f,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    LeftArrow = 0x25,
    UpArrow = 0x26,
    RightArrow = 0x27,
    DownArrow = 0x28,
    Select = 0x29,
    Print = 0x2a,
    Execute = 0x2b,
    PrintScreen = 0x2c,
    Insert = 0x2d,
    Delete = 0x2e,
    Help = 0x2f,
    LMeta = 0x5b,
    RMeta = 0x5c,
    Apps = 0x5d,
    Sleep = 0x5f,
    Numpad0 = 0x60,
    Numpad1 = 0x61,
    Numpad2 = 0x62,
    Numpad3 = 0x63,
    Numpad4 = 0x64,
    Numpad5 = 0x65,
    Numpad6 = 0x66,
    Numpad7 = 0x67,
    Numpad8 = 0x68,
    Numpad9 = 0x69,
    Multiply = 0x6a,
    Add = 0x6b,
    Separator = 0x6c,
    Subtract = 0x6d,
    Decimal = 0x6e,
    Divide = 0x6f,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,
    Numlock = 0x90,
    Scroll = 0x91,
    LShift = 0xa0,
    RShift = 0xa1,
    LControl = 0xa2,
    RControl = 0xa3,
    LAlt = 0xa4,
    RAlt = 0xa5,
    BrowserBack = 0xa6,
    BrowserForward = 0xa7,
    BrowserRefresh = 0xa8,
    BrowserStop = 0xa9,
    BrowserSearch = 0xaa,
    BrowserFavorites = 0xab,
    BrowserHome = 0xac,
    VolumeMute = 0xad,
    VolumeDown = 0xae,
    VolumeUp = 0xaf,
    MediaNextTrack = 0xb0,
    MediaPrevTrack = 0xb1,
    MediaStop = 0xb2,
    MediaPlayPause = 0xb3,
    LaunchMail = 0xb4,
    LaunchMediaSelect = 0xb5,
    LaunchApp1 = 0xb6,
    LaunchApp2 = 0xb7,
    Processkey = 0xe5,
    Play = 0xfa,
    Zoom = 0xfb,
);

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VirtualKey {
    Back,
    Tab,
    Clear,
    Return,
    Shift,
    Control,
    Alt,
    Pause,
    CapsLock,
    KanaOrHangul,
    ImeOn,
    Junja,
    Final,
    KanjiOrHanja,
    ImeOff,
    Escape,
    ImeConvert,
    ImeNonconvert,
    ImeAccept,
    ImeModechange,
    Space,
    PageUp,
    PageDown,
    End,
    Home,
    LeftArrow,
    UpArrow,
    RightArrow,
    DownArrow,
    Select,
    Print,
    Execute,
    PrintScreen,
    Insert,
    Delete,
    Help,
    LMeta,
    RMeta,
    Apps,
    Sleep,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Numlock,
    Scroll,
    LShift,
    RShift,
    LControl,
    RControl,
    LAlt,
    RAlt,
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    VolumeMute,
    VolumeDown,
    VolumeUp,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaPlayPause,
    LaunchMail,
    LaunchMediaSelect,
    LaunchApp1,
    LaunchApp2,
    Processkey,
    Play,
    Zoom,
}
