use crate::event::EventProcessor;

pub struct Handle {}

impl Handle {
    pub fn register(_: EventProcessor) -> Self {
        todo!("See akl-native-lib-prototype Handle::register")
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        todo!("See akl-native-lib-prototype Handle::unregister")
    }
}
