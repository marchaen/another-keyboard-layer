//! Processes low level keyboard events on top of the platform independent event
//! and event change request abstraction.
//!
//! The native keyboard hook creates events by using the specific os apis to
//! translate and hook into the message system for keyboard input. The events
//! then get passed along to the [`event processor`](EventProcessor::process())
//! who decides if and how the event should be changed. Those changes are
//! applied by the keyboard hook, it then fetches the next message and repeats
//! this procedure.

use std::collections;

use crate::{
    key::{Key, KeyCombination},
    Configuration,
};

/// The action that caused this event which is either the pressing or releasing
/// of any keyboard key.
#[derive(Debug, Clone, Copy)]
enum Action {
    Press,
    Release,
}

/// Platform independent abstraction over a low level keyboard event that
/// specifies the trigger and related [`key`](crate::key::Key).
#[derive(Debug, Clone, Copy)]
struct Event {
    pub action: Action,
    pub key: Key,
}

/// Platform independent abstraction over actions that are taken in response to
/// processing an event such as blocking or replacing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseAction {
    DoNothing,
    Block,
    ReplaceWith(KeyCombination),
}

/// Processes events according to the algorithm visualized in the **README**.
pub struct EventProcessor {
    switch_key: Key,
    default_combination: Option<KeyCombination>,
    mappings: collections::HashMap<KeyCombination, KeyCombination>,
    currently_pressed: Vec<Key>,
    block_events: bool,
    key_combination_executed: bool,
}

/// Convenience implementation for creating an event processor with the specific
/// configuration which will fail if the `switch_key` field is none.
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
    /// Process the event as specified in the **README**.
    pub fn process(&mut self, event: Event) -> ResponseAction {
        match event.action {
            Action::Press => {
                if event.key == self.switch_key {
                    self.block_events = true;
                    self.currently_pressed.clear();
                    return ResponseAction::Block;
                }

                if !self.block_events {
                    return ResponseAction::DoNothing;
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
                        return ResponseAction::ReplaceWith(
                            *replacement_combination,
                        );
                    }
                }

                ResponseAction::Block
            }
            Action::Release => {
                if event.key == self.switch_key {
                    self.block_events = false;

                    if !self.key_combination_executed {
                        if let Some(combination) = self.default_combination {
                            return ResponseAction::ReplaceWith(combination);
                        }
                    }

                    self.key_combination_executed = false;
                    return ResponseAction::Block;
                }

                if let Ok(index) =
                    self.currently_pressed.binary_search(&event.key)
                {
                    self.currently_pressed.swap_remove(index);

                    if self.block_events {
                        return ResponseAction::Block;
                    }
                }

                ResponseAction::DoNothing
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

        test_event!(Action::Press, 'a', ResponseAction::DoNothing);

        test_event!(Action::Press, switch_key, ResponseAction::Block);
        test_event!(Action::Release, 'a', ResponseAction::DoNothing);

        test_event!(Action::Press, 'a', ResponseAction::Block);
        test_event!(Action::Release, 'a', ResponseAction::Block);

        test_event!(
            Action::Release,
            switch_key,
            ResponseAction::ReplaceWith(default_combination)
        );

        test_event!(Action::Press, switch_key, ResponseAction::Block);

        test_event!(Action::Press, 't', ResponseAction::ReplaceWith(kc!('a')));

        test_event!(Action::Release, switch_key, ResponseAction::Block);
    }
}
