//! See the ffi module for usage instructions and general documentation.

mod ffi;
mod handle;

use handle::AklHandle;

use num_enum::TryFromPrimitive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AklError {
    #[error("The switch key has to be some value before starting akl.")]
    NotInitialized,
    #[error("Akl is already running.")]
    AlreadyRunning,
    #[error("Akl was already stopped.")]
    AlreadyStopped,
}

#[derive(Default, Clone)]
pub struct Configuration {
    pub switch_key: Option<Key>,
    pub default_combination: Option<KeyCombination>,
    pub mappings: std::collections::HashMap<KeyCombination, KeyCombination>,
}

#[derive(Default)]
pub struct AnotherKeyboardLayer {
    pub configuration: Configuration,
    handle: Option<AklHandle>,
}

impl AnotherKeyboardLayer {
    fn is_not_initialized(&self) -> bool {
        matches!(self.configuration.switch_key, None)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.handle, Some(_))
    }

    pub fn start(&mut self) -> Result<(), AklError> {
        if self.is_not_initialized() {
            return Err(AklError::NotInitialized);
        }

        if self.is_running() {
            return Err(AklError::AlreadyRunning);
        }

        let mut handle = AklHandle::new(self.configuration.clone());
        handle.register();
        self.handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), AklError> {
        if !self.is_running() {
            return Err(AklError::AlreadyStopped);
        }

        let mut handle = self.handle.take().unwrap();
        handle.unregister();

        Ok(())
    }
}

impl Drop for AnotherKeyboardLayer {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Text(char),
    Virtual(VirtualKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyCombination(Key, Option<Key>, Option<Key>, Option<Key>);

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
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
    Kana,
    Hangul,
    ImeOn,
    Junja,
    Final,
    Hanja,
    Kanji,
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
