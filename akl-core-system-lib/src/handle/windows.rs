use crate::Configuration;

use log::info;

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
        KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
        WM_SYSKEYUP,
    },
};

pub struct AklHandle {
    configuration: Configuration,
    hook: Option<HHOOK>,
}

// TODO: Remove println statements
impl AklHandle {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            configuration,
            hook: None,
        }
    }

    pub fn register(&mut self) {
        if self.hook.is_some() {
            self.unregister();
        }

        info!("Registering hook...");

        let register_result = unsafe {
            // Details and safety see:
            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(raw_event_listener),
                HMODULE(0),
                0,
            )
        };

        match register_result {
            Ok(hook) => self.hook = Some(hook),
            Err(error) => panic!(
                "Trying to register a global keyboard event listener failed: {} ({})", 
                error.message().to_string_lossy(),
                error.code()
            )
        }
    }

    pub fn unregister(&mut self) {
        if let Some(hook) = self.hook {
            info!("Unregistering hook...");
            // It doesn't matter for us if this fails, because the application stops
            // anyway or tries to re register and gets a detailed error there.
            let _ = unsafe {
                // Details and safety see:
                // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
                UnhookWindowsHookEx(hook)
            };

            self.hook = None;
        }
    }
}

impl Drop for AklHandle {
    fn drop(&mut self) {
        self.unregister();
    }
}

// See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
// for invariants and other important information about the implementation of
// this type of callback.
unsafe extern "system" fn raw_event_listener(
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

    match wparam.0 as u32 {
        WM_KEYDOWN => println!("KEYDOWN: {} ({})", event.scanCode, event.vkCode),
        WM_KEYUP => println!("KEYUP: {} ({})", event.scanCode, event.vkCode),
        WM_SYSKEYDOWN => println!("SYSKEYDOWN: {} ({})", event.scanCode, event.vkCode),
        WM_SYSKEYUP => println!("SYSKEYUP: {} ({})", event.scanCode, event.vkCode),
        _ => unreachable!("See values for wparam here: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc")
    }

    return CallNextHookEx(HHOOK(0), code, wparam, lparam);
}
