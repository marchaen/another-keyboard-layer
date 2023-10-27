mod debugger;

use std::ffi::c_void;

use windows::Win32::{
    Foundation::{BOOL, HMODULE, LPARAM, LRESULT, WPARAM},
    System::SystemServices::{
        DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
    },
    UI::WindowsAndMessaging::{
        CallNextHookEx, HHOOK, MSG, WM_CHAR, WM_DEADCHAR, WM_SYSCHAR, WM_SYSDEADCHAR,
    },
};

use debugger::Debugger;

// Export a dll entry point function to initialize the global debugger.
// See https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-entry-point-function
#[allow(non_snake_case)]
#[no_mangle]
unsafe extern "system" fn DllMain(
    _dllModule: HMODULE,
    fdwReason: u32,
    _lpReserved: *const c_void,
) -> BOOL {
    #[allow(clippy::wildcard_in_or_patterns)]
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            Debugger::init();
        }
        DLL_PROCESS_DETACH => {
            Debugger::destroy();
        }
        DLL_THREAD_ATTACH | DLL_THREAD_DETACH | _ => (),
    }

    true.into()
}

// See https://learn.microsoft.com/en-us/windows/win32/winmsg/getmsgproc
#[no_mangle]
unsafe extern "system" fn raw_translated_character_hook(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // As documented we can't handle any events that have a code lower than zero.
    // We should instead pass them to the next hook and return their result.
    if code < 0 {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }

    let message_pointer: *mut MSG = std::mem::transmute(lparam);
    let message = message_pointer.as_mut().unwrap();

    #[allow(clippy::wildcard_in_or_patterns)]
    match message.message {
        // See https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-char
        WM_CHAR => {
            // Use native endianness because the bytes are just reinterpreted
            // as u16s on the same system and not stored or send anywhere.
            let raw_param_bytes = message.wParam.0.to_ne_bytes();

            // Safety: The pointer size on modern windows is always 64 bit
            // (8 bytes) so converting them to four u16s is safe.
            let (_, raw_utf16_code_points, _) = raw_param_bytes.align_to::<u16>();

            let translated_character = char::decode_utf16(raw_utf16_code_points.iter().copied())
                .find_map(Result::ok)
                .unwrap();

            Debugger::write(&format!("Translated character: \"{translated_character}\""));
        }
        // Other possible events that aren't relevant for this library's use
        // case but are generally related to the wm_char messages.
        // See https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#character-messages
        WM_DEADCHAR | WM_SYSCHAR | WM_SYSDEADCHAR | _ => (),
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}
