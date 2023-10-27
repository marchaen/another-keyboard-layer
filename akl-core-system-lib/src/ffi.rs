//! # FFI for AKL.Common
//! 
//! This module does all the conversions which are needed to be able to interact
//! with the actual library from c#. This also makes it possible for the core
//! lib to completely ignore that any of this ffiying is even happening which
//! means the writing of the core lib is a lot more comfortable.

/// Pointer type for methods that require an instance of
/// [AnotherKeyboardLayer](crate::AnotherKeyboardLayer) to make sure no data can
/// be corrupted from the c# site easily. Creating a pointer type isn't required
/// for ffi but makes the api a bit nicer because there aren't any `*void` used.
#[repr(C)]
pub struct AklContext;

/// A ffi safe representation of a [key](crate::Key) which is used to transfer
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

/// Indicates the type of key stored in [FfiKey].
#[repr(u8)]
pub enum FfiKeyKind {
    /// A single character such as 'a', 'ü' or 'è'
    Text,
    /// Any key that doesn't produce text when pressed. 
    /// See also [VirtualKey](crate::VirtualKey) in the implementation.
    Virtual,
    /// No key at all, unfortunately ffi doesn't allow us to represent a key
    /// combination with `Option`-types so this is the easiest solution for
    /// safely transferring [key combinations](crate::KeyCombination) with less
    /// than four keys from c#.
    None
}

/// Ffi save representation of a [key combination](crate::KeyCombination) which
/// uses the [FfiKeyKind::None] variant to represent an undefined / missing key.
/// 
/// **Caution**: This struct can represent an invalid key combination if all
/// keys are set to none by the caller.
#[repr(C)]
pub struct FfiKeyCombination(FfiKey, FfiKey, FfiKey, FfiKey);

/// Ffi save result type that contains an error message as a cstring if the
/// `has_error` field is set to true.
#[repr(C)]
pub struct FfiResult {
    /// Indicates that an operation has failed.
    has_error: bool,
    /// Contains information about why the operation failed.
    error_message: *mut i8,
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
    unimplemented!()
}

/// Destroys the akl context by dropping it. This means the Drop implementation
/// will be run and thus deactivate another keyboard layer.
#[no_mangle]
pub extern "C" fn destroy(raw_context: *mut AklContext) {
    unimplemented!()
}

/// Tries to start another keyboard layer. Fails if the switch key wasn't set
/// yet or if it is already running. See 
/// [start](crate::AnotherKeyboardLayer::start)-method of AnotherKeyboardLayer.
#[no_mangle]
pub extern "C" fn start(raw_context: *mut AklContext) -> FfiResult {
    unimplemented!()
}

/// Tries to stop another keyboard layer. Fails if the virtual layer isn't
/// running.
#[no_mangle]
pub extern "C" fn stop(raw_context: *mut AklContext) -> FfiResult {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn set_switch_key(
    raw_context: *mut AklContext,
    new_key: FfiKey,
) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn set_default_combination(
    raw_context: *mut AklContext,
    key_combination: FfiKeyCombination,
) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn add_mapping(
    raw_context: *mut AklContext,
    target: FfiKeyCombination,
    replacement: FfiKeyCombination,
) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn remove_mapping(
    raw_context: *mut AklContext,
    target: FfiKeyCombination,
) -> FfiResult {
    unimplemented!()
}
