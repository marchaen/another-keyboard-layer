#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

#[cfg(not(target_os = "windows"))]
use xkeysym::Keysym as X11Key;

macro_rules! virtual_key_code_to_virtual_key_translations {
    ($($name: ident: win = $windows_translation: expr, x11 = $($x11_translation: ident $(|)?)+),*,) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum VirtualKey {
            $($name),*
        }

        impl TryFrom<&str> for VirtualKey {
            type Error = ();

            fn try_from(name: &str) -> Result<Self, Self::Error> {
                match name {
                    $(stringify!($name) => Ok(VirtualKey::$name),)*
                    _ => Err(())
                }
            }
        }

        #[cfg(target_os = "windows")]
        impl TryFrom<VIRTUAL_KEY> for VirtualKey {
            type Error = ();

            fn try_from(windows_key: VIRTUAL_KEY) -> Result<Self, Self::Error> {
                match windows_key.0 {
                    $($windows_translation => Ok(VirtualKey::$name),)*
                    _ => Err(()),
                }
            }
        }

        #[cfg(target_os = "windows")]
        impl From<VirtualKey> for VIRTUAL_KEY {
            fn from(virtual_key: VirtualKey) -> Self {
                match virtual_key {
                    $(VirtualKey::$name => VIRTUAL_KEY($windows_translation),)*
                }
            }
        }

        #[cfg(target_os = "windows")]
        impl VirtualKey {
            pub fn to_windows_key(self) -> u16 {
                Into::<VIRTUAL_KEY>::into(self).0
            }
        }

        #[cfg(not(target_os = "windows"))]
        impl TryFrom<X11Key> for VirtualKey {
            type Error = ();

            fn try_from(x11_key: X11Key) -> Result<Self, Self::Error> {
                match x11_key {
                    $($(X11Key::$x11_translation => Ok(VirtualKey::$name)),*),*,
                    _ => Err(())
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        impl From<VirtualKey> for X11Key {
            fn from(virtual_key: VirtualKey) -> Self {
                match virtual_key {
                    $(VirtualKey::$name => [$(X11Key::$x11_translation),*][0]),*
                }
            }
        }
    };
}

virtual_key_code_to_virtual_key_translations!(
    Back: win = 0x08, x11 = BackSpace,
    Tab: win = 0x09, x11 = Tab | KP_Tab,
    Clear: win = 0x0c, x11 = Clear,
    Return: win = 0x0d, x11 = Return,
    Pause: win = 0x13, x11 = Pause,
    CapsLock: win = 0x14, x11 = Caps_Lock,
    Escape: win = 0x1b, x11 = Escape,
    Space: win = 0x20, x11 = space | KP_Space,
    PageUp: win = 0x21, x11 = Page_Up | KP_Page_Up,
    PageDown: win = 0x22, x11 = Page_Down | KP_Page_Down,
    Home: win = 0x24, x11 = Home | KP_Home,
    End: win = 0x23, x11 = End | KP_End,
    LeftArrow: win = 0x25, x11 = Left | KP_Left,
    UpArrow: win = 0x26, x11 = Up | KP_Up,
    RightArrow: win = 0x27, x11 = Right | KP_Right,
    DownArrow: win = 0x28, x11 = Down | KP_Down,
    Select: win = 0x29, x11 = Select,
    Print: win = 0x2a, x11 = Print,
    Execute: win = 0x2b, x11 = Execute,
    Insert: win = 0x2d, x11 = Insert | KP_Insert,
    Delete: win = 0x2e, x11 = Delete | KP_Delete,
    Help: win = 0x2f, x11 = Help,
    LMeta: win = 0x5b, x11 = Meta_L,
    RMeta: win = 0x5c, x11 = Meta_R,
    Apps: win = 0x5d, x11 = Menu,
    Sleep: win = 0x5f, x11 = XF86_Sleep,
    Numpad0: win = 0x60, x11 = KP_0,
    Numpad1: win = 0x61, x11 = KP_1,
    Numpad2: win = 0x62, x11 = KP_2,
    Numpad3: win = 0x63, x11 = KP_3,
    Numpad4: win = 0x64, x11 = KP_4,
    Numpad5: win = 0x65, x11 = KP_5,
    Numpad6: win = 0x66, x11 = KP_6,
    Numpad7: win = 0x67, x11 = KP_7,
    Numpad8: win = 0x68, x11 = KP_8,
    Numpad9: win = 0x69, x11 = KP_9,
    Multiply: win = 0x6a, x11 = KP_Multiply,
    Add: win = 0x6b, x11 = KP_Add,
    Separator: win = 0x6c, x11 = KP_Separator,
    Subtract: win = 0x6d, x11 = KP_Subtract,
    Decimal: win = 0x6e, x11 = KP_Decimal,
    Divide: win = 0x6f, x11 = KP_Divide,
    F1: win = 0x70, x11 = F1 | KP_F1,
    F2: win = 0x71, x11 = F2 | KP_F2,
    F3: win = 0x72, x11 = F3 | KP_F3,
    F4: win = 0x73, x11 = F4 | KP_F4,
    F5: win = 0x74, x11 = F5,
    F6: win = 0x75, x11 = F6,
    F7: win = 0x76, x11 = F7,
    F8: win = 0x77, x11 = F8,
    F9: win = 0x78, x11 = F9,
    F10: win = 0x79, x11 = F10,
    F11: win = 0x7A, x11 = F11,
    F12: win = 0x7B, x11 = F12,
    F13: win = 0x7C, x11 = F13,
    F14: win = 0x7D, x11 = F14,
    F15: win = 0x7E, x11 = F15,
    F16: win = 0x7F, x11 = F16,
    F17: win = 0x80, x11 = F17,
    F18: win = 0x81, x11 = F18,
    F19: win = 0x82, x11 = F19,
    F20: win = 0x83, x11 = F20,
    F21: win = 0x84, x11 = F21,
    F22: win = 0x85, x11 = F22,
    F23: win = 0x86, x11 = F23,
    F24: win = 0x87, x11 = F24,
    Numlock: win = 0x90, x11 = Num_Lock,
    Scroll: win = 0x91, x11 = Scroll_Lock,
    LShift: win = 0xa0, x11 = Shift_L,
    RShift: win = 0xa1, x11 = Shift_R,
    LControl: win = 0xa2, x11 = Control_L,
    RControl: win = 0xa3, x11 = Control_R,
    LAlt: win = 0xa4, x11 = Alt_L,
    RAlt: win = 0xa5, x11 = Alt_R,
    BrowserBack: win = 0xa6, x11 = XF86_Back,
    BrowserForward: win = 0xa7, x11 = XF86_Forward,
    BrowserRefresh: win = 0xa8, x11 = XF86_Refresh,
    BrowserStop: win = 0xa9, x11 = XF86_Stop,
    BrowserSearch: win = 0xaa, x11 = XF86_Search,
    BrowserFavorites: win = 0xab, x11 = XF86_Favorites,
    BrowserHome: win = 0xac, x11 = XF86_HomePage,
    VolumeMute: win = 0xad, x11 = XF86_AudioMute,
    VolumeDown: win = 0xae, x11 = XF86_AudioLowerVolume,
    VolumeUp: win = 0xaf, x11 = XF86_AudioRaiseVolume,
    MediaNextTrack: win = 0xb0, x11 = XF86_AudioNext,
    MediaPrevTrack: win = 0xb1, x11 = XF86_AudioPrev,
    MediaStop: win = 0xb2, x11 = XF86_AudioStop,
    MediaPlayPause: win = 0xb3, x11 = XF86_AudioPlay,
    LaunchMail: win = 0xb4, x11 = XF86_Mail,
    LaunchApp1: win = 0xb6, x11 = XF86_Launch1,
    LaunchApp2: win = 0xb7, x11 = XF86_Launch2,
    Play: win = 0xfa, x11 = XF86_AudioRandomPlay,
);
