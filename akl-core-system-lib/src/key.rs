#![allow(non_upper_case_globals)]

use std::hash::Hash;

use num_enum::TryFromPrimitive;
use thiserror::Error;

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

/// Represents a single key. Not very useful by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Text(char),
    Virtual(VirtualKey),
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Self::Text(value)
    }
}

impl From<VirtualKey> for Key {
    fn from(value: VirtualKey) -> Self {
        Self::Virtual(value)
    }
}

/// Represents a valid key combination as used by the event processor to
/// translate mappings.
///
/// The keys have to be stored without `None` values in between each other
/// (`self.4` can't be `Some(key)` if `self.3` is `None`). This requirement
/// can't be represented with the type system and has to be manually checked for
/// at every place where a `KeyCombination` is created instead.
#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub struct KeyCombination(Key, Option<Key>, Option<Key>, Option<Key>);

#[derive(Error, Debug, PartialEq, Eq)]
pub enum KeyCombinationConversionError {
    #[error("Each key combination has to start with one valid key.")]
    NotEnoughKeys,
    #[error("A key combination can be made of maximum four keys.")]
    TooManyKeys,
}

impl KeyCombination {
    fn count_keys(&self) -> u8 {
        if self.1.is_some() {
            return 2;
        }

        if self.2.is_some() {
            return 3;
        }

        if self.3.is_some() {
            return 4;
        }

        return 1;
    }
}

impl TryFrom<&[Option<Key>]> for KeyCombination {
    type Error = KeyCombinationConversionError;

    fn try_from(value: &[Option<Key>]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(KeyCombinationConversionError::NotEnoughKeys);
        }

        if value.len() > 4 {
            return Err(KeyCombinationConversionError::TooManyKeys);
        }

        if value[0].is_none() {
            return Err(KeyCombinationConversionError::NotEnoughKeys);
        }

        // Used to filter any none values even if they are at the end of the
        // slice. With this the key combination is guaranteed to not contain
        // another key after a `None` value in itself.
        let mut filtered_keys =
            value.iter().skip(1).filter_map(|key| key.as_ref().copied());

        Ok(Self(
            value[0].unwrap(),
            filtered_keys.next(),
            filtered_keys.next(),
            filtered_keys.next(),
        ))
    }
}

impl From<&KeyCombination> for [Option<Key>; 4] {
    fn from(value: &KeyCombination) -> Self {
        [Some(value.0), value.1, value.2, value.3]
    }
}

impl Hash for KeyCombination {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: [Option<Key>; 4] = self.into();
        keys.sort();
        keys.into_iter().for_each(|key| key.hash(state));
    }
}

impl PartialEq for KeyCombination {
    fn eq(&self, other: &Self) -> bool {
        if self.count_keys() != other.count_keys() {
            return false;
        }

        let self_slice: [Option<Key>; 4] = self.into();
        let other_slice: [Option<Key>; 4] = other.into();

        self_slice.iter().all(|key| other_slice.contains(key))
    }
}

impl Eq for KeyCombination {}

macro_rules! define_virtual_key_codes {
    ($($name: ident = $windows_translation: expr),*,) => {
        /// Represents any key that doesn't produce any text / characters when
        /// pressed, dead keys excluded.
        ///
        /// Based on [windows virtual key codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
        /// modified to be platform agnostic and with clearer names.
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
        #[repr(u8)]
        pub enum VirtualKey {
            $($name,)*
        }

        #[derive(Error, Debug, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum VirtualKeyConversionError {
            #[error("No virtual key with the specified name exists.")]
            NoKeyWithSpecifiedName,
            #[cfg(target_os = "windows")]
            #[error("No virtual key with the specified code ({:X}) exists.", (.0).0)]
            NoKeyWithSpecifiedCode(VIRTUAL_KEY),
        }

        impl TryFrom<&str> for VirtualKey {
            type Error = VirtualKeyConversionError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $(stringify!($name) => Ok(VirtualKey::$name),)*
                    _ => Err(VirtualKeyConversionError::NoKeyWithSpecifiedName)
                }
            }
        }

        #[cfg(target_os = "windows")]
        impl TryFrom<VIRTUAL_KEY> for VirtualKey {
            type Error = VirtualKeyConversionError;

            fn try_from(windows_key: VIRTUAL_KEY) -> Result<Self, Self::Error> {
                match windows_key.0 {
                    $($windows_translation => Ok(VirtualKey::$name),)*
                    _ => Err(VirtualKeyConversionError::NoKeyWithSpecifiedCode(windows_key)),
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
    };
}

// Defines the virtual key code enum. For now the macro only declares a windows
// translation from here https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
// this will change when the linux implementation get created.
define_virtual_key_codes!(
    Back = 0x08,
    Tab = 0x09,
    Clear = 0x0c,
    Return = 0x0d,
    Shift = 0x10,
    Control = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    // Kana key on Japanese and Hangul key on Korean keyboards
    KanaOrHangul = 0x15,
    ImeOn = 0x16,
    Junja = 0x17,
    Final = 0x18,
    // Kanji key on Japanese and Hanja key on Korean keyboards
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

#[cfg(test)]
mod tests {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    #[cfg(target_os = "windows")]
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB;

    use super::*;

    // Static key constants that are guaranteed to be valid
    const KEY_A: Key = Key::Text('a');
    const KEY_B: Key = Key::Text('b');
    const KEY_ESCAPE: Key = Key::Virtual(VirtualKey::Escape);
    const KEY_RETURN: Key = Key::Virtual(VirtualKey::Return);

    #[test]
    fn test_key_conversions() {
        assert_eq!(Into::<Key>::into('a'), KEY_A);
        assert_eq!(Into::<Key>::into(VirtualKey::Escape), KEY_ESCAPE);
    }

    #[test]
    fn test_key_combination_conversions() {
        assert_eq!(
            Err(KeyCombinationConversionError::NotEnoughKeys),
            TryInto::<KeyCombination>::try_into([].as_slice())
        );

        assert_eq!(
            Err(KeyCombinationConversionError::NotEnoughKeys),
            TryInto::<KeyCombination>::try_into([None].as_slice())
        );

        assert_eq!(
            Ok(KeyCombination(KEY_A, None, None, None)),
            TryInto::<KeyCombination>::try_into([Some(KEY_A)].as_slice())
        );

        assert_eq!(
            [Some(KEY_A), Some(KEY_ESCAPE), None, None],
            Into::<[Option<Key>; 4]>::into(&KeyCombination(
                KEY_A,
                Some(KEY_ESCAPE),
                None,
                None
            ))
        );

        assert_eq!(
            [Some(KEY_A), Some(KEY_ESCAPE), Some(KEY_B), None],
            Into::<[Option<Key>; 4]>::into(&KeyCombination(
                KEY_A,
                Some(KEY_ESCAPE),
                Some(KEY_B),
                None
            ))
        );

        assert_eq!(
            [Some(KEY_A), Some(KEY_ESCAPE), Some(KEY_B), Some(KEY_RETURN)],
            Into::<[Option<Key>; 4]>::into(&KeyCombination(
                KEY_A,
                Some(KEY_ESCAPE),
                Some(KEY_B),
                Some(KEY_RETURN)
            ))
        );

        assert_eq!(
            Err(KeyCombinationConversionError::TooManyKeys),
            TryInto::<KeyCombination>::try_into(
                [None, None, None, None, None].as_slice()
            )
        );
    }

    #[test]
    fn test_key_combination_hash_and_eq() {
        let first_hash = {
            let mut hasher = DefaultHasher::new();
            KeyCombination(KEY_A, Some(KEY_ESCAPE), Some(KEY_RETURN), None)
                .hash(&mut hasher);
            hasher.finish()
        };

        let second_hash = {
            let mut hasher = DefaultHasher::new();
            KeyCombination(KEY_A, Some(KEY_RETURN), Some(KEY_ESCAPE), None)
                .hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(first_hash, second_hash);

        assert_eq!(
            KeyCombination(
                KEY_B,
                Some(KEY_A),
                Some(KEY_ESCAPE),
                Some(KEY_RETURN)
            ),
            KeyCombination(
                KEY_ESCAPE,
                Some(KEY_B),
                Some(KEY_RETURN),
                Some(KEY_A)
            ),
        )
    }

    #[test]
    fn test_virtual_key_conversion() {
        assert_eq!(Ok(VirtualKey::Tab), TryInto::<VirtualKey>::try_into("Tab"));

        assert_eq!(
            Err(VirtualKeyConversionError::NoKeyWithSpecifiedName),
            TryInto::<VirtualKey>::try_into("")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_virtual_key_conversion_windows() {
        assert_eq!(Ok(VirtualKey::Tab), VK_TAB.try_into());
        assert_eq!(
            Err(VirtualKeyConversionError::NoKeyWithSpecifiedCode(
                VIRTUAL_KEY(u16::MAX)
            )),
            TryInto::<VirtualKey>::try_into(VIRTUAL_KEY(u16::MAX))
        );

        assert_eq!(VK_TAB, VirtualKey::Tab.into());
        assert_eq!(VirtualKey::Tab.to_windows_key(), VK_TAB.0);
    }
}
