use windows::Win32::UI::{
    Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, VIRTUAL_KEY},
    WindowsAndMessaging::KBDLLHOOKSTRUCT,
};

pub fn translate_to_character(event: &KBDLLHOOKSTRUCT) -> Option<char> {
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

pub fn windows_to_virtual_key(windows_key: u16) -> Option<VirtualKey> {
    VIRTUAL_KEY(windows_key).try_into().ok()
}

// TODO: Add this impl block to the current implementation of the library
impl VirtualKey {
    #[cfg(target_os = "windows")]
    pub fn to_windows_key(self) -> u16 {
        Into::<windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY>::into(self).0
    }
}

macro_rules! windows_virtual_key_code_to_virtual_key_translations {
    ($($name: ident = $translation: expr),*,) => {
        impl TryFrom<VIRTUAL_KEY> for VirtualKey {
            type Error = ();

            fn try_from(windows_key: VIRTUAL_KEY) -> Result<Self, Self::Error> {
                match windows_key.0 {
                    $($translation => Ok(VirtualKey::$name),)*
                    _ => Err(()),
                }
            }
        }

        impl From<VirtualKey> for VIRTUAL_KEY {
            fn from(virtual_key: VirtualKey) -> Self {
                match virtual_key {
                    $(VirtualKey::$name => VIRTUAL_KEY($translation),)*
                }
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
