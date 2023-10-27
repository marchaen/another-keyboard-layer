use std::mem::size_of;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY,
    },
    WindowsAndMessaging::GetMessageExtraInfo,
};

use crate::{debugger::Debugger, translation::VirtualKey};

fn key_to_input(key: VirtualKey, should_release: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key.into(),
                dwExtraInfo: unsafe { GetMessageExtraInfo() }.0 as usize,
                dwFlags: if should_release {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS::default()
                },
                ..Default::default()
            },
        },
    }
}

pub fn type_virtual_key(key: VirtualKey) {
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
    let result = unsafe {
        SendInput(
            &[key_to_input(key, false), key_to_input(key, true)],
            size_of::<INPUT>() as i32,
        )
    };
    Debugger::write(&format!("Typed virtual keys: {result}"));
}

// See also https://stackoverflow.com/questions/22291282/using-sendinput-to-send-unicode-characters-beyond-uffff
fn char_to_input(key: char, should_release: bool) -> (INPUT, Option<INPUT>) {
    let mut flags = KEYEVENTF_UNICODE;

    if should_release {
        flags |= KEYEVENTF_KEYUP;
    }

    let mut encoded_code_points_buffer = [0u16; 2];
    let encoded_code_points = key.encode_utf16(&mut encoded_code_points_buffer);

    let first_input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: encoded_code_points[0],
                dwFlags: flags,
                dwExtraInfo: unsafe { GetMessageExtraInfo() }.0 as usize,
                ..Default::default()
            },
        },
    };

    let second_input = if encoded_code_points.len() == 2 {
        Some(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: encoded_code_points[1],
                    dwFlags: flags,
                    dwExtraInfo: unsafe { GetMessageExtraInfo() }.0 as usize,
                    ..Default::default()
                },
            },
        })
    } else {
        None
    };

    (first_input, second_input)
}

pub fn type_char_key(key: char) {
    let mut inputs = Vec::with_capacity(4);

    let press_inputs = char_to_input(key, false);
    inputs.push(press_inputs.0);

    if let Some(input) = press_inputs.1 {
        inputs.push(input);
    }

    let release_inputs = char_to_input(key, true);
    inputs.push(release_inputs.0);

    if let Some(input) = release_inputs.1 {
        inputs.push(input);
    }

    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
    let result = unsafe { SendInput(inputs.as_slice(), size_of::<INPUT>() as i32) };

    Debugger::write(&format!("Typed char keys: {result}"));
}
