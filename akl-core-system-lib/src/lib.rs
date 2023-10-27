//! See the ffi module for usage instructions and general documentation.

mod ffi;
mod handle;
mod key;

use std::collections;

use thiserror::Error;

use handle::AklHandle;
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
    pub mappings: collections::HashMap<KeyCombination, KeyCombination>,
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

#[derive(Debug, Clone, Copy)]
enum Action {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
struct Event {
    pub action: Action,
    pub key: Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeEventRequest {
    None,
    Block,
    ReplaceWith(KeyCombination),
}

struct EventProcessor {
    switch_key: Key,
    default_combination: Option<KeyCombination>,
    mappings: collections::HashMap<KeyCombination, KeyCombination>,
    currently_pressed: Vec<Key>,
    block_events: bool,
    key_combination_executed: bool,
}

impl From<Configuration> for EventProcessor {
    fn from(value: Configuration) -> Self {
        Self {
            switch_key: value
                .switch_key
                .expect("Switch key should be valid for an event processor."),
            default_combination: value.default_combination,
            mappings: value.mappings,
            currently_pressed: vec![],
            block_events: false,
            key_combination_executed: false,
        }
    }
}

impl EventProcessor {
    pub fn process(&mut self, event: Event) -> ChangeEventRequest {
        match event.action {
            Action::Press => {
                if event.key == self.switch_key {
                    self.block_events = true;
                    self.currently_pressed.clear();
                    return ChangeEventRequest::Block;
                }

                if !self.block_events {
                    return ChangeEventRequest::None;
                }

                self.currently_pressed.push(event.key);

                let maybe_target_combination: Result<KeyCombination, _> =
                    self.currently_pressed.as_slice().try_into();

                if let Ok(target_combination) = maybe_target_combination {
                    if let Some(replacement_combination) =
                        self.mappings.get(&target_combination)
                    {
                        self.key_combination_executed = true;
                        self.currently_pressed.pop();
                        return ChangeEventRequest::ReplaceWith(
                            *replacement_combination,
                        );
                    }
                }

                ChangeEventRequest::Block
            }
            Action::Release => {
                if event.key == self.switch_key {
                    self.block_events = false;

                    if !self.key_combination_executed {
                        if let Some(combination) = self.default_combination {
                            return ChangeEventRequest::ReplaceWith(
                                combination,
                            );
                        }
                    }

                    self.key_combination_executed = false;
                    return ChangeEventRequest::Block;
                }

                if let Ok(index) =
                    self.currently_pressed.binary_search(&event.key)
                {
                    self.currently_pressed.swap_remove(index);

                    if self.block_events {
                        return ChangeEventRequest::Block;
                    }
                }

                ChangeEventRequest::None
            }
        }
    }
}

