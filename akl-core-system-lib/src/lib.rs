//! Abstracts interactions with the native virtual layer in a high level easy
//! to use interface.
//!
//! This abstraction completely manages the lifecycle of virtual layers that
//! means making sure associated resources get released when not needed anymore
//! as well as passing along the current configuration when ever the virtual
//! layer should be started (which means a new one gets created and replaces the
//! current one).
//!
//! See [`AnotherKeyboardLayer::new()`] to get started.
//!
//! # Ffi
//!
//! The [`ffi`](`crate::ffi`)-module exposes the interface which the c#
//! `AKL.Common` package uses to implement all relevant functionality that the
//! virtual layer needs. The akl.common package and ffi module are highly
//! coupled and have to be modified together whenever one of them changes.
//! Unfortunately this is also the case for the [`virtual key`](crate::key::VirtualKey).
#![allow(rustdoc::private_intra_doc_links)]

mod event;
mod ffi;
mod handle;
mod key;

use std::collections;

use thiserror::Error;

use handle::AklHandle;
use key::{Key, KeyCombination};

/// Represents any errors that can occur while interacting with the virtual
/// layer.
#[derive(Error, Debug)]
pub enum AklError {
    #[error("The switch key has to be some value before starting akl.")]
    NotConfigured,
    #[error("Akl is already running.")]
    AlreadyRunning,
    #[error("Akl was already stopped.")]
    AlreadyStopped,
}

/// Configuration that is needed for the virtual layer to work.
#[derive(Default, Clone)]
pub struct Configuration {
    /// Key that when pressed makes the virtual layer start to listen for key
    /// bindings and block all events from reaching any windows.
    pub switch_key: Option<Key>,
    /// Default key combination that is invoked when the switch key is pressed
    /// and released without executing any key bindings.
    pub default_combination: Option<KeyCombination>,
    /// Defines the target and replacement key bindings which are matched
    /// against while the switch key is pressed.
    pub mappings: collections::HashMap<KeyCombination, KeyCombination>,
}

/// High level abstraction over the interactions with the underlying platform
/// specific virtual layer.
pub struct AnotherKeyboardLayer {
    pub configuration: Configuration,
    handle: Option<AklHandle>,
}

impl AnotherKeyboardLayer {
    /// Creates a new akl that has to be configured before [`starting`](Self::start())
    /// it. Information about the configuration [`here`](crate::Configuration).
    fn new() -> Self {
        #[cfg(debug_assertions)]
        {
            use log::LevelFilter;
            use simplelog::{ConfigBuilder, ThreadLogMode, WriteLogger};

            if let Ok(connection) =
                std::net::TcpStream::connect("127.0.0.1:7777")
            {
                let config = {
                    match ConfigBuilder::new()
                        .set_thread_level(LevelFilter::Error)
                        .set_thread_mode(ThreadLogMode::Both)
                        .set_target_level(LevelFilter::Error)
                        .set_time_offset_to_local()
                    {
                        Ok(config_builder) | Err(config_builder) => {
                            config_builder.build()
                        }
                    }
                };

                // Errors if the global logger is already initialized. Can be
                // ignored safely without any consequences.
                let _ =
                    WriteLogger::init(LevelFilter::Trace, config, connection);
            }
        }

        Self {
            configuration: Default::default(),
            handle: Default::default(),
        }
    }

    /// Checks if the virtual layer is configured correctly. For a correct
    /// configuration at least the [`switch_key`](Configuration::switch_key) has
    /// to be set.
    #[must_use]
    pub fn is_not_configured(&self) -> bool {
        matches!(self.configuration.switch_key, None)
    }

    /// Checks if the native platform specific virtual layer is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.handle, Some(_))
    }

    /// Starts the native virtual layer with a copy of the configuration.
    ///
    /// # Errors
    ///
    /// - [`AklError::NotConfigured`] => If [`is_not_configured()`](Self::is_not_configured())
    /// returns `true`
    /// - [`AklError::AlreadyRunning`] => If [`is_running()`](Self::is_running())
    /// returns `true`
    pub fn start(&mut self) -> Result<(), AklError> {
        if self.is_not_configured() {
            return Err(AklError::NotConfigured);
        }

        if self.is_running() {
            return Err(AklError::AlreadyRunning);
        }

        let mut handle = AklHandle::new(self.configuration.clone());
        handle.register();
        self.handle = Some(handle);

        Ok(())
    }

    /// Stops the currently running native virtual layer.
    ///
    /// # Errors
    ///
    /// - [`AklError::AlreadyStopped`] => If [`is_running`](Self::is_running())
    /// returns `false`
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

/// Drop implementation that explicitly calls [`stop()`](Self::stop()) to make
/// sure any resources associated with the native virtual layer get released.
impl Drop for AnotherKeyboardLayer {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}
