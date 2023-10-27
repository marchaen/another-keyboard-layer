//! Linux mock implementation to pass compilation

use thiserror::Error;

use crate::event::EventProcessor;

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Can't have two hooks registered at the same time.")]
    AnotherHookIsAlreadyInstalled,
    #[error("{0}")]
    RegistrationFailed(String),
}

pub struct Handle {
}

impl Handle {
    pub fn register(
        associated_event_processor: EventProcessor,
    ) -> Result<Self, HandleError> {
        unimplemented!()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unimplemented!()
    }
}
