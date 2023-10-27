//! Translation of windows native to and from platform independent
//! representations of keys, events and input data.

use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, INPUT, INPUT_0,
        INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY,
    },
    WindowsAndMessaging::{
        GetMessageExtraInfo, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use crate::{
    event::{Action, Event},
    key::{Key, KeyCombination, VirtualKey},
};

/// Translates the windows native keyboard input event to an abstract platform
/// independent [`event`](crate::event::Event) which can further be processed
/// by an [`event processor`](crate::event::EventProcessor).
///
/// See also [`to_character`] which is used if the parsing of a [`virtual key`](crate::key::VirtualKey)
/// fails and the [`unicode replacement character`](https://compart.com/en/unicode/U+FFFD)
/// which is set as the event key if that also fails.
pub fn to_abstract_event(action: u32, event: &KBDLLHOOKSTRUCT) -> Event {
    let action = match action {
        WM_KEYDOWN | WM_SYSKEYDOWN=> Action::Press,
        WM_KEYUP | WM_SYSKEYUP => Action::Release,
        _ => unreachable!("See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc#wparam-in"),
    };

    // Try to translate the character from the keyboard event or use the unicode
    // replacement character "�" (https://compart.com/en/unicode/U+FFFD).
    let key = TryInto::<VirtualKey>::try_into(VIRTUAL_KEY(event.vkCode as u16))
        .map_or_else(
            |_| to_character(event).unwrap_or('\u{FFFD}').into(),
            Into::into,
        );

    Event { action, key }
}

/// Tries to translate the keyboard input event to a possibly corresponding text
/// representation of the key that triggered the event.
///
/// If a key doesn't produce any text when pressed such as Escape or Shift
/// none will be returned.
///
/// If the conclusion of the translation is that multiple text characters are
/// produced as a result of the event, the first decoded character inserted by
/// [ToUnicodeEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex)
/// into the output buffer will be returned.
fn to_character(event: &KBDLLHOOKSTRUCT) -> Option<char> {
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardstate
    let mut keyboard_state = [0u8; 256];

    let state_retrieval_result =
        unsafe { GetKeyboardState(&mut keyboard_state) };

    // Getting the state failed and thus translating the event is impossible.
    if state_retrieval_result.0 == 0 {
        return None;
    }

    // Retrieving the keyboard layout takes on average four micro seconds on a
    // relatively low powered device so even though the documentation talks
    // about caching this information that just currently isn't worth it.
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardlayout
    let keyboard_layout = unsafe { GetKeyboardLayout(0) };

    // The documentation doesn't specify how many code points can be returned
    // max for one key event so 8 was chosen arbitrary.
    let mut unicode_code_point_buffer = [0u16; 8];

    // Setting the third bit of the flags value makes sure to keep the keyboard
    // state as is. Not doing this breaks the dead characters (^, `, ´).
    let flags = 0b100;

    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex
    let translation_result = unsafe {
        ToUnicodeEx(
            event.vkCode,
            event.scanCode,
            &keyboard_state,
            &mut unicode_code_point_buffer,
            flags,
            keyboard_layout,
        )
    };

    if translation_result == 0 {
        return None;
    }

    let unicode_code_points = unicode_code_point_buffer
        [..translation_result.unsigned_abs() as usize]
        .iter()
        .copied();

    // Even if more multiple characters were translated only return the first one
    char::decode_utf16(unicode_code_points).find_map(Result::ok)
}

/// Translate a key combination to native input events that can be send using
/// the [`SendInput`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
/// windows api method.
///
/// Unfortunately a [text key](`Key::Text`) can not be simulated with only one
/// native input event if this api should be able to handle all possible values
/// that the char type can represent.
///
/// Futhermore each `key down` event needs a corresponding `key up` event to
/// correctly simulate a key press, so the number of required events doubles
/// again landing on 16 total possible events that are sent for each key
/// combination.
pub fn to_native_input_events(
    key_combination: KeyCombination,
) -> [Option<INPUT>; 16] {
    let mut inputs: [Option<INPUT>; 16] = [None; 16];
    let mut down_input_counter = 0;
    let mut up_input_counter = 15;

    Into::<[Option<Key>; 4]>::into(&key_combination)
        .iter()
        .flatten()
        .for_each(|key| match *key {
            Key::Text(character) => {
                // key down events
                let (down_input, maybe_down_input) =
                    character_to_input(character, InputAction::KeyDown);

                inputs[down_input_counter] = Some(down_input);
                down_input_counter += 1;

                if maybe_down_input.is_some() {
                    inputs[down_input_counter] = maybe_down_input;
                    down_input_counter += 1;
                }

                // key up events
                let (up_input, maybe_up_input) =
                    character_to_input(character, InputAction::KeyUp);

                inputs[up_input_counter] = Some(up_input);
                up_input_counter -= 1;

                if maybe_up_input.is_some() {
                    inputs[up_input_counter] = maybe_up_input;
                    up_input_counter -= 1;
                }
            }
            Key::Virtual(virtual_key) => {
                inputs[down_input_counter] = Some(virtual_key_to_input(
                    virtual_key,
                    InputAction::KeyDown,
                ));
                down_input_counter += 1;

                inputs[up_input_counter] =
                    Some(virtual_key_to_input(virtual_key, InputAction::KeyUp));
                up_input_counter -= 1;
            }
        });

    inputs
}

/// Safe representation of key up and key down events that is also used to
/// create the correct [`dwFlags`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput)
/// for each action.
#[derive(Clone, Copy)]
enum InputAction {
    KeyUp,
    KeyDown,
}

impl InputAction {
    /// Either no flags or only the key up flag.
    fn to_flags(self) -> KEYBD_EVENT_FLAGS {
        match self {
            Self::KeyUp => KEYEVENTF_KEYUP,
            Self::KeyDown => KEYBD_EVENT_FLAGS::default(),
        }
    }

    /// Same as [`to_flags`](Self::to_flags) but adds the flag needed for any
    /// unicode input.
    fn to_unicode_flags(self) -> KEYBD_EVENT_FLAGS {
        self.to_flags() | KEYEVENTF_UNICODE
    }
}

/// Creates a native keyboard input with the action for the virtual key.
fn virtual_key_to_input(key: VirtualKey, input_action: InputAction) -> INPUT {
    // See also:
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-input
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key.into(),
                dwExtraInfo: unsafe { GetMessageExtraInfo() }.0 as usize,
                dwFlags: input_action.to_flags(),
                ..Default::default()
            },
        },
    }
}

/// Creates native keyboard inputs needed to simulate pressing or releasing
/// the character.
///
/// See also <https://stackoverflow.com/questions/22291282/using-sendinput-to-send-unicode-characters-beyond-uffff>
/// for an explanation on why multiple events are only needed for characters.
fn character_to_input(
    key: char,
    input_action: InputAction,
) -> (INPUT, Option<INPUT>) {
    let mut encoded_code_points_buffer = [0u16; 2];
    let encoded_code_points = key.encode_utf16(&mut encoded_code_points_buffer);

    let first_input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: encoded_code_points[0],
                dwFlags: input_action.to_unicode_flags(),
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
                    dwFlags: input_action.to_unicode_flags(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_native_input_events() {
        // kc => KeyCombination
        macro_rules! kc {
            ($($key: expr $(,)?)*) => {
                TryInto::<KeyCombination>::try_into([$(Into::<Key>::into($key)), *].as_slice())
                    .expect("Static key combination should always be valid.")
            };
        }

        // Used only for counting the number of arguments in the macro invocation
        // See https://danielkeep.github.io/tlborm/book/blk-counting.html
        macro_rules! replace_expr {
            ($_e:tt $sub:expr) => {
                $sub
            };
        }

        macro_rules! test_event_generation {
            ($($key: expr $(,)?)*) => {
                let number_of_keys = <[()]>::len(&[$(replace_expr!($key ())),*]);
                let inputs = to_native_input_events(kc!($($key), *));

                assert_eq!(
                    inputs
                        .iter()
                        .flatten()
                        .count(),
                    number_of_keys * 2
                );

                for input in &inputs[0..number_of_keys] {
                    assert!(input.is_some())
                }

                for input in &inputs[inputs.len() - number_of_keys..] {
                    assert!(input.is_some())
                }
            };
        }

        // Static key constants that are guaranteed to be valid
        const KEY_A: Key = Key::Text('a');
        const KEY_B: Key = Key::Text('b');
        const KEY_ESCAPE: Key = Key::Virtual(VirtualKey::Escape);
        const KEY_RETURN: Key = Key::Virtual(VirtualKey::Return);

        test_event_generation!(KEY_A);
        test_event_generation!(KEY_A, KEY_ESCAPE);
        test_event_generation!(KEY_A, KEY_ESCAPE, KEY_B);
        test_event_generation!(KEY_A, KEY_ESCAPE, KEY_B, KEY_RETURN);
    }

    #[test]
    fn test_virtual_key_to_input() {
        macro_rules! test_virtual_key_to_input {
            ($key: expr, $action: expr) => {
                let input = virtual_key_to_input($key, $action);

                assert_eq!(input.r#type, INPUT_KEYBOARD);

                unsafe {
                    assert_eq!(input.Anonymous.ki.wVk, $key.into());
                    assert_eq!(input.Anonymous.ki.wScan, 0);
                    assert_eq!(input.Anonymous.ki.dwFlags, $action.to_flags());
                }
            };
        }

        test_virtual_key_to_input!(VirtualKey::CapsLock, InputAction::KeyDown);
        test_virtual_key_to_input!(VirtualKey::CapsLock, InputAction::KeyUp);
    }

    #[test]
    fn test_character_to_input() {
        macro_rules! test_character_to_input {
            ($character: expr, $action: expr) => {
                let flags = $action.to_unicode_flags();

                let mut decoded = [0u16; 2];
                $character.encode_utf16(&mut decoded);

                let (first, second) = character_to_input($character, $action);

                assert_eq!(first.r#type, INPUT_KEYBOARD);

                unsafe {
                    assert_eq!(first.Anonymous.ki.wVk, VIRTUAL_KEY(0));
                    assert_eq!(first.Anonymous.ki.wScan, decoded[0]);
                    assert_eq!(first.Anonymous.ki.dwFlags, flags);
                }

                if decoded[1] == 0 {
                    assert!(second.is_none());
                } else {
                    let second = second.unwrap();

                    assert_eq!(second.r#type, INPUT_KEYBOARD);

                    unsafe {
                        assert_eq!(second.Anonymous.ki.wVk, VIRTUAL_KEY(0));
                        assert_eq!(second.Anonymous.ki.wScan, decoded[1]);
                        assert_eq!(second.Anonymous.ki.dwFlags, flags);
                    }
                }
            };
        }

        test_character_to_input!('a', InputAction::KeyDown);
        test_character_to_input!('a', InputAction::KeyUp);

        test_character_to_input!('😊', InputAction::KeyUp);
        test_character_to_input!('😊', InputAction::KeyDown);
    }
}
