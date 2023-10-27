use std::mem::size_of;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY,
    },
    WindowsAndMessaging::GetMessageExtraInfo,
};

use crate::{
    debugger::Debugger,
    events::{Key, KeyCombination},
    translation::VirtualKey,
};

pub fn send_key_combination(key_combination: KeyCombination) {
    let keys: [Option<Key>; 4] = key_combination.into();
    let mut send_keys = 0;

    for key in &keys {
        if key.is_none() {
            break;
        }

        send_keys += 1;
        Debugger::write(&format!("Simulating key {key:?}"));
        input_key(key.unwrap(), false);
    }

    while send_keys != 0 {
        send_keys -= 1;
        input_key(keys[send_keys].unwrap(), true);
    }
}

// TODO: If this method will be made available to other modules directly change
// the usage of the boolean with an enum Action::Press / Action::Release usage.
fn input_key(key: Key, should_release: bool) {
    match key {
        Key::Text(character) => {
            let raw_inputs = char_to_input(character, should_release);

            let mut inputs = [raw_inputs.0; 2];
            let mut input_counter = 1;

            if let Some(input) = raw_inputs.1 {
                input_counter += 1;
                inputs[1] = input;
            }

            // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
            let _ = unsafe { SendInput(&inputs[..input_counter], size_of::<INPUT>() as i32) };
        }
        Key::Virtual(virtual_key) => {
            // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
            let _ = unsafe {
                SendInput(
                    &[key_to_input(virtual_key, should_release)],
                    size_of::<INPUT>() as i32,
                )
            };
        }
    }
}

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
