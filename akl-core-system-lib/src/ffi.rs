//! # FFI for c# package AKL.Common
//!
//! This module does all the conversions which are needed to be able to interact
//! with the actual library from c#. This also makes it possible for the core
//! lib to completely ignore that any of this ffiying is even happening which
//! means the writing of the core lib is a lot more comfortable.

// The dead code is used from the language that is interfacing with this library.
#![allow(dead_code)]

use crate::{
    key::Key, key::KeyCombination, key::VirtualKey, AnotherKeyboardLayer,
};

/// Pointer type for methods that require an instance of
/// [`AnotherKeyboardLayer`](crate::AnotherKeyboardLayer) to make sure no data
/// can be corrupted from the c# site easily. Creating a pointer type isn't
/// required for ffi but makes the api a bit nicer because there aren't any
/// `*void` used.
#[repr(C)]
pub struct AklContext;

/// Convenience function to get a mutable reference from a raw context.
///
/// # Safety
///
/// The returned reference has an arbitrary lifetime but is only valid for as
/// long as the raw context is also valid. Never hold on to this reference for a
/// longer time than the corresponding pointer. (Although holding one doesn't
/// mean the underlying memory can't be deallocated and thus cause ub anyway.)
#[inline(always)]
fn akl_from_raw<'arbitrary>(
    raw: *mut AklContext,
) -> &'arbitrary mut AnotherKeyboardLayer {
    unsafe { &mut *raw.cast::<AnotherKeyboardLayer>() }
}

/// A ffi safe representation of a [`key`](crate::Key) which is used to transfer
/// from the c# key type safely.
#[repr(C)]
pub struct FfiKey {
    /// Replacement for an actual utf-8 char because they aren't ffi safe.
    text: u32,
    /// Representation of a virtual key that is shared between c# and rust.
    /// The enum definition has to be kept the same so that this doesn't break.
    named: u8,
    /// Marks the type of key this instance represents.
    kind: FfiKeyKind,
}

/// Indicates the type of key stored in [`FfiKey`].
#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum FfiKeyKind {
    /// A single character such as 'a', 'ü' or 'è'
    Text,
    /// Any key that doesn't produce text when pressed.
    /// See also [VirtualKey](crate::key::VirtualKey) in the implementation.
    Virtual,
    /// No key at all, unfortunately ffi doesn't allow us to represent a key
    /// combination with `Option`-types so this is the easiest solution for
    /// safely transferring [key combinations](crate::KeyCombination) with less
    /// than four keys from c#.
    None,
}

impl TryFrom<FfiKey> for Key {
    type Error = ();

    fn try_from(value: FfiKey) -> Result<Self, Self::Error> {
        match value.kind {
            FfiKeyKind::Text => {
                let parsing_result = char::from_u32(value.text);

                if let Some(parsed_char) = parsing_result {
                    return Ok(Key::Text(parsed_char));
                }

                Err(())
            }
            FfiKeyKind::Virtual => {
                let parsing_result = VirtualKey::try_from(value.named);

                if let Ok(virtual_key) = parsing_result {
                    return Ok(Key::Virtual(virtual_key));
                }

                Err(())
            }
            FfiKeyKind::None => Err(()),
        }
    }
}

/// Ffi save representation of a [key combination](crate::KeyCombination) which
/// uses the [`FfiKeyKind::None`] variant to represent an undefined / missing
/// key.
///
/// **Caution**: This struct can represent an invalid key combination if all
/// keys are set to none by the caller.
#[repr(C)]
pub struct FfiKeyCombination(FfiKey, FfiKey, FfiKey, FfiKey);

impl TryFrom<FfiKeyCombination> for KeyCombination {
    type Error = ();

    fn try_from(value: FfiKeyCombination) -> Result<Self, Self::Error> {
        let first_key: Key = {
            let first_key = value.0.try_into();

            if first_key.is_err() {
                return Err(());
            }

            first_key.unwrap()
        };

        [
            Some(first_key),
            value.1.try_into().ok(),
            value.2.try_into().ok(),
            value.3.try_into().ok(),
        ]
        .as_slice()
        .try_into()
        .map_err(|_| ())
    }
}

/// Ffi save result type that contains an error message as a cstring if the
/// `has_error` field is set to true.
#[repr(C)]
pub struct FfiResult {
    /// Indicates that an operation has failed.
    has_error: bool,
    /// Contains information about why the operation failed.
    error_message: *mut i8,
}

impl FfiResult {
    fn ok() -> Self {
        Self {
            has_error: false,
            error_message: std::ptr::null::<i8>().cast_mut(),
        }
    }

    fn error(message: &str) -> Self {
        let message = std::ffi::CString::new(message)
            .expect("Every &str is a valid c string.");

        Self {
            has_error: true,
            error_message: message.into_raw(),
        }
    }
}

/// Deallocates the error message of an ffi result. There is unfortunately no
/// other way than for the c# side to pass the message back to rust just for
/// deallocation.
///
/// This also means that the program will leak memory if this method doesn't get
/// called for each created error message.
#[no_mangle]
pub extern "C" fn destroy_error_message(error_message: *mut i8) {
    unsafe { std::ffi::CString::from_raw(error_message) };
}

/// Allocates an akl context which can further be used to setup and start
/// another keyboard layer.
///
/// Has to be passed back to rust for deallocation. See [destroy]
#[no_mangle]
pub extern "C" fn init() -> *mut AklContext {
    Box::into_raw(Box::new(AnotherKeyboardLayer::new())).cast::<AklContext>()
}

/// Destroys the akl context by dropping it. This means the Drop implementation
/// will be run and thus deactivate another keyboard layer.
#[no_mangle]
pub extern "C" fn destroy(raw_context: *mut AklContext) {
    let _ =
        unsafe { Box::from_raw(raw_context.cast::<AnotherKeyboardLayer>()) };
}

/// Tries to start another keyboard layer. Fails if the switch key wasn't set
/// yet or if it is already running. See
/// [start](crate::AnotherKeyboardLayer::start)-method of `AnotherKeyboardLayer`.
#[no_mangle]
pub extern "C" fn start(raw_context: *mut AklContext) -> FfiResult {
    let akl = akl_from_raw(raw_context);

    let result = akl.start();

    if let Err(error) = result {
        return FfiResult::error(&error.to_string());
    }

    FfiResult::ok()
}

/// Tries to stop another keyboard layer. Fails if the virtual layer isn't
/// running.
#[no_mangle]
pub extern "C" fn stop(raw_context: *mut AklContext) -> FfiResult {
    let akl = akl_from_raw(raw_context);

    let result = akl.stop();

    if let Err(error) = result {
        return FfiResult::error(&error.to_string());
    }

    FfiResult::ok()
}

/// Check if the virtual layer is running.
#[no_mangle]
pub extern "C" fn is_running(raw_context: *mut AklContext) -> bool {
    akl_from_raw(raw_context).is_running()
}

/// Tries to set the switch key. Fails if the key [kind](FfiKeyKind) is `None`.
#[no_mangle]
pub extern "C" fn set_switch_key(
    raw_context: *mut AklContext,
    new_key: FfiKey,
) -> FfiResult {
    if new_key.kind == FfiKeyKind::None {
        return FfiResult::error("Can't set switch key to none.");
    }

    let akl = akl_from_raw(raw_context);
    let parsed_key: Result<Key, _> = new_key.try_into();

    if let Ok(key) = parsed_key {
        akl.configuration.switch_key = Some(key);
        FfiResult::ok()
    } else {
        FfiResult::error("Trying to parse the key failed (invalid value).")
    }
}

/// Sets the default key combination. A key combination with all keys set to
/// [None](FfiKeyKind::None) means no default key combination.
#[no_mangle]
pub extern "C" fn set_default_combination(
    raw_context: *mut AklContext,
    key_combination: FfiKeyCombination,
) {
    let akl = akl_from_raw(raw_context);
    let parsed_combination: Result<KeyCombination, _> =
        key_combination.try_into();

    if let Ok(combination) = parsed_combination {
        akl.configuration.default_combination = Some(combination);
    } else {
        akl.configuration.default_combination = None;
    }
}

/// Adds a mapping or overrides it if it is already targeted. Can fail if any
/// of the key combinations are invalid.
#[no_mangle]
pub extern "C" fn add_mapping(
    raw_context: *mut AklContext,
    target: FfiKeyCombination,
    replacement: FfiKeyCombination,
) -> FfiResult {
    let (target, replacement) = {
        let target = target.try_into();

        if target.is_err() {
            return FfiResult::error("The target key combination is invalid.");
        }

        let replacement = replacement.try_into();

        if replacement.is_err() {
            return FfiResult::error(
                "The replacement key combination is invalid.",
            );
        }

        (target.unwrap(), replacement.unwrap())
    };

    let akl = akl_from_raw(raw_context);
    let _ = akl.configuration.mappings.insert(target, replacement);

    FfiResult::ok()
}

/// Removes the mapping with the specified target. Regardless of if the key
/// combination is valid only a return value of `true` means that a combination
/// was removed.
#[no_mangle]
pub extern "C" fn remove_mapping(
    raw_context: *mut AklContext,
    target: FfiKeyCombination,
) -> bool {
    let target = {
        let target = target.try_into();

        if target.is_err() {
            return false;
        }

        target.unwrap()
    };

    let akl = akl_from_raw(raw_context);
    let previous = akl.configuration.mappings.remove(&target);

    previous.is_some()
}

/// Clears all mappings. Doesn't update the currently running layer.
#[no_mangle]
pub extern "C" fn clear_mappings(raw_context: *mut AklContext) {
    akl_from_raw(raw_context).configuration.mappings.clear();
}
