use crate::Configuration;

// TODO: Implement linux handle
pub struct AklHandle {
    configuration: Configuration,
}

impl AklHandle {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            configuration
        }
    }

    pub fn register(&mut self) {

    }

    pub fn unregister(&mut self) {

    }
}
