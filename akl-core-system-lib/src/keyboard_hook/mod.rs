//! Native low level platform dependent keyboard input hook abstraction that
//! directly calls the [`event processor`](crate::event::EventProcessor).
//!
//! This module just reexports the correct keyboard hook implementation
//! depending on the `target_os` attribute. For documentation open the
//! implementation module directly.

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::{Handle, HandleError};

#[cfg(not(target_os = "windows"))]
mod linux;

#[cfg(not(target_os = "windows"))]
pub use self::linux::{Handle, HandleError};
