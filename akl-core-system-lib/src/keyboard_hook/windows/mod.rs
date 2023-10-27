//! Windows implementation of the keyboard hook using the native
//! [WH_KEYBOARD_LL](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks#wh_keyboard_ll)
//! hook.
//!
//! Using only the exposed api guarantees no undefined behavior and severe logic
//! bugs (because of uncaught events that should have been processed).

mod translation;

use std::{
    mem,
    os::windows::prelude::AsRawHandle,
    ptr,
    sync::{mpsc, Mutex},
    thread,
};

use windows::Win32::{
    Foundation::{HANDLE, HMODULE, LPARAM, LRESULT, WPARAM},
    System::Threading::GetThreadId,
    UI::{
        Input::KeyboardAndMouse::{SendInput, INPUT},
        WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
            UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
            WM_APP,
        },
    },
};

use log::{error, info};
use thiserror::Error;

use crate::event::{EventProcessor, ResponseAction};

/// All errors that can occur while trying to register a keyboard hook.
#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Can't have two hooks registered at the same time.")]
    AnotherHookIsAlreadyInstalled,
    #[error("{0}")]
    RegistrationFailed(String),
}

/// Windows keyboard hook handle implementation which ensures safety.
///
/// This handle enforces all invariants that could cause undefined behavior or
/// at least severe logic bugs when broken, by only exposing a safe
/// [`register`](Handle::register) function to obtain a keyboard hook handle.
///
/// Also handles the cleanup of the message queue associated with the [`ManagedHook`].
pub struct Handle {
    // Used to keep the managed hook alive until the handle gets destroyed.
    #[allow(dead_code)]
    hook: ManagedHook,
    message_queue_thread: u32,
}

impl Handle {
    /// Tries to register a keyboard hook with its associated event processor
    /// and starts a message queue to process the messages.
    ///
    /// # Errors
    ///
    /// Registration can fail if another keyboard hook is currently already
    /// registered or if the windows [`set hook`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
    /// call fails.
    pub fn register(
        associated_event_processor: EventProcessor,
    ) -> Result<Self, HandleError> {
        let (keyboard_hook_sender, keyboard_hook_receiver) = mpsc::channel();

        let message_queue = thread::spawn(move || {
            // Important: The hook has to be registered from the same thread in
            // which the message queue is running. That's why there is a need
            // to explicitly send the handle to the main thread.
            let _ = keyboard_hook_sender
                .send(ManagedHook::register(associated_event_processor));
            drop(keyboard_hook_sender);

            start_message_queue();
        });

        let thread_id = {
            // How to convert std library handle to windows-rs handle: https://stackoverflow.com/a/73574560
            let thread_handle = HANDLE(message_queue.as_raw_handle() as isize);

            // See https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadid
            unsafe { GetThreadId(thread_handle) }
        };

        let keyboard_hook = keyboard_hook_receiver.recv().expect(
            "Should not drop the sender before receiving the keyboard hook.",
        );

        // Prevent a message queue from not being stopped when the keyboard hook
        // registration fails.
        if keyboard_hook.is_err() {
            Self::stop_message_queue(thread_id);
        }

        Ok(Self {
            hook: keyboard_hook?,
            message_queue_thread: thread_id,
        })
    }

    /// Internal function used to terminate the message queue safely.
    fn stop_message_queue(thread_id: u32) {
        info!("Stop message queue {}", thread_id);

        // See post thread message https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew
        let result = unsafe {
            PostThreadMessageW(thread_id, WM_APP + 1, WPARAM(0), LPARAM(0))
        };

        info!("Stop message queue result {result:?}");
    }
}

/// Terminate the message queue associated with the raw hook.
impl Drop for Handle {
    fn drop(&mut self) {
        Self::stop_message_queue(self.message_queue_thread);
    }
}

/// Blocks the current thread until a message is received.
///
/// The first call to
/// [`GetMessage`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage)
/// or [`PeekMessage`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew)
/// internally creates a message queue for the current thread this behavior is
/// expected.
fn start_message_queue() {
    info!("Running message queue and block until end.");

    let mut message = MSG::default();

    // We don't need to handle the return type we only want to listen to the
    // first message that gets posted anyway.
    //
    // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage
    let result = unsafe { GetMessageW(ptr::addr_of_mut!(message), None, 0, 0) };

    info!(
        "Got message shuting down message queue {1:#X} (Status: {0})",
        result.0, message.message
    );
}

/// Wrapper around a native hook handle that manages the resources associated
/// with it such as the event processor and makes sure they get cleaned up when
/// this hook gets destroyed.
///
/// Without additionally starting a message queue registering this hook won't do
/// anything. See [`Handle::register`]
struct ManagedHook(HHOOK);

impl ManagedHook {
    /// Tries to register a keyboard hook with the event processor.
    ///
    /// # Errors
    ///
    /// Registration can fail if another keyboard hook is currently already
    /// registered or if the windows [`set hook`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
    /// call fails.
    pub fn register(
        associated_event_processor: EventProcessor,
    ) -> Result<Self, HandleError> {
        info!("Register global keyboard listener hook.");

        let mut keyboard_hook_event_processor = EVENT_PROCESSOR
            .lock()
            .expect("Global hook doesn't panic so it can't poison the mutex");

        if keyboard_hook_event_processor.is_some() {
            return Err(HandleError::AnotherHookIsAlreadyInstalled);
        }

        keyboard_hook_event_processor.replace(associated_event_processor);

        let register_result = unsafe {
            // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(raw_keyboard_input_hook),
                HMODULE(0),
                0,
            )
        };

        match register_result {
            Ok(hook) => {
                info!(
                    "Successfully registered the global keyboard listener hook ({hook:?})."
                );
                Ok(Self(hook))
            }
            Err(error) => {
                // Remove the global event processor when registration fails.
                let _ = keyboard_hook_event_processor.take();

                Err(HandleError::RegistrationFailed(format!(
                    "Trying to register a global keyboard listener failed: {} ({})",
                    error.message().to_string_lossy(),
                    error.code()
                )))
            }
        }
    }
}

/// Unregisters the hook and makes sure all relevant resources get cleaned up.
impl Drop for ManagedHook {
    fn drop(&mut self) {
        info!("Unregister global raw keyboard listener hook");

        let result = unsafe {
            // See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
            UnhookWindowsHookEx(self.0)
        };

        // Drop associated event processor
        EVENT_PROCESSOR
            .lock()
            .expect("Global hook doesn't panic so it can't poison the mutex")
            .take();

        // Safety: Being able to lock the event processor means the keyboard
        // input hook has finished it's last execution and won't get called
        // another time because it was unregistered and thus accessing
        // CURRENTLY_WRITING is safe.
        //
        // See UnhookWindowsHookEx documentation above for guarantee remark.
        unsafe { CURRENTLY_WRITING = false };

        info!("Unregister global raw keyboard listener result: {result:?}",);
    }
}

/// The event processor currently associated with the raw keyboard input hook.
static EVENT_PROCESSOR: Mutex<Option<EventProcessor>> = Mutex::new(None);

/// The raw keyboard input hook also receives events that it causes. This flag
/// is used to ignore any events that occur while sending input events.
static mut CURRENTLY_WRITING: bool = false;

/// See microsoft documentation on [lowlevelkeyboardproc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc).
unsafe extern "system" fn raw_keyboard_input_hook(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let default_behavior = || CallNextHookEx(HHOOK(0), code, wparam, lparam);

    // As documented we can't handle any events that have a code lower than zero.
    // We should instead pass them to the next hook and return their result.
    if code < 0 {
        return default_behavior();
    }

    // Safety: The raw keyboard_input_hook is always called from the same thread
    // that registered it and [`Handle::register`] takes care of ensuring only
    // one raw keyboard input hook gets registered which guarantees exclusive
    // access to CURRENTLY_WRITING.
    if unsafe { CURRENTLY_WRITING } {
        return default_behavior();
    }

    let mut event_processor = EVENT_PROCESSOR
        .lock()
        .expect("Raw keyboard input hook never panics and thus never poisons this mutex.");

    if event_processor.is_none() {
        error!("Invalid global state for raw keyboard input hook. (No associated event processor)");
        return default_behavior();
    }

    // Safety: We do the check right above and return early if the even processor is none.
    let event_processor = event_processor.as_mut().unwrap_unchecked();

    // See https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc#lparam-in
    let event_pointer: *const KBDLLHOOKSTRUCT = mem::transmute(lparam);

    let event =
        translation::to_abstract_event(wparam.0 as u32, &*event_pointer);
    let change_request = event_processor.process(event);

    info!("{event:?} => {change_request:?}");

    match change_request {
        ResponseAction::Block => LRESULT(1),
        ResponseAction::ReplaceWith(key_combination) => {
            // Safety: See above
            unsafe {
                CURRENTLY_WRITING = true;
            }

            for input in translation::to_native_input_events(key_combination)
                .into_iter()
                .flatten()
            {
                SendInput(&[input], mem::size_of::<INPUT>() as i32);
            }

            // Safety: See above
            unsafe {
                CURRENTLY_WRITING = false;
            }
            LRESULT(1)
        }
        ResponseAction::DoNothing => default_behavior(),
    }
}
