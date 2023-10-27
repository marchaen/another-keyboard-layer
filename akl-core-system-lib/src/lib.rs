//! See the ffi module for usage instructions and general documentation.

mod ffi;
mod handle;
mod key;

use handle::AklHandle;

use thiserror::Error;
use key::{Key, KeyCombination};

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

    #[must_use]
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

        // Handle can't be none when self.is_running returns true because the
        // implementation is just checking that handle is Some(_).
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
