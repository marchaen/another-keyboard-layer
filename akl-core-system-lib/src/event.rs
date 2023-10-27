use std::collections;

use crate::{
    key::{Key, KeyCombination},
    Configuration,
};

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

#[cfg(test)]
mod tests {
    use crate::key::VirtualKey;

    use super::*;

    #[test]
    fn test_event_processor() {
        // Test that the event processor works exactly as visualized in the
        // "Kern" section of the README.

        // kc => KeyCombination
        macro_rules! kc {
            ($($key: expr $(,)?)*) => {
                TryInto::<KeyCombination>::try_into([$(Into::<Key>::into($key)), *].as_slice())
                    .expect("Static key combination should always be valid.")
            };
        }

        let switch_key = Key::Virtual(VirtualKey::Space);
        let default_combination = kc!(VirtualKey::Return);
        let mappings = collections::HashMap::from([(kc!('t'), kc!('a'))]);

        let mut event_processor: EventProcessor = {
            let mut config = Configuration::default();

            config.switch_key = Some(switch_key);
            config.default_combination = Some(default_combination);
            config.mappings = mappings;

            config.into()
        };

        macro_rules! test_event {
            ($action: expr, $key: expr, $change: expr) => {
                assert_eq!(
                    event_processor.process(Event {
                        action: $action,
                        key: $key.into()
                    }),
                    $change
                );
            };
        }

        test_event!(Action::Press, 'a', ChangeEventRequest::None);

        test_event!(Action::Press, switch_key, ChangeEventRequest::Block);
        test_event!(Action::Release, 'a', ChangeEventRequest::None);

        test_event!(Action::Press, 'a', ChangeEventRequest::Block);
        test_event!(Action::Release, 'a', ChangeEventRequest::Block);

        test_event!(
            Action::Release,
            switch_key,
            ChangeEventRequest::ReplaceWith(default_combination)
        );

        test_event!(Action::Press, switch_key, ChangeEventRequest::Block);

        test_event!(
            Action::Press,
            't',
            ChangeEventRequest::ReplaceWith(kc!('a'))
        );

        test_event!(Action::Release, switch_key, ChangeEventRequest::Block);
    }
}
