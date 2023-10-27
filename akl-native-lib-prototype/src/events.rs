use std::{collections, hash::Hash};

use crate::{debugger::Debugger, translation::VirtualKey};

// The key and key combination definition won't live in the events module
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Text(char),
    Virtual(VirtualKey),
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Self::Text(value)
    }
}

impl From<VirtualKey> for Key {
    fn from(value: VirtualKey) -> Self {
        Self::Virtual(value)
    }
}

impl From<&'static str> for Key {
    fn from(value: &'static str) -> Self {
        if let Ok(virtual_key) = value.try_into() {
            Self::Virtual(virtual_key)
        } else {
            Self::Text(value.chars().next().expect(
                "Should always be able to create a virtual or char / text key from static strings.",
            ))
        }
    }
}

#[macro_export]
macro_rules! key_combination {
    ($($key: literal $(+)?)*) => {
        TryInto::<KeyCombination>::try_into([$($key.into()), *].as_slice())
            .expect("Static key combination should always be valid.")
    };
}

// TODO: Custom equality operations for allowing that the keys are pressed in
// any order and not just the one which was specified in the configuration.
//
// Example: "Shift+j" => The user could press j first and then shift and the
// key combination wouldn't be recognized
#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub struct KeyCombination(Key, Option<Key>, Option<Key>, Option<Key>);

impl KeyCombination {
    fn count_keys(&self) -> u8 {
        let mut count = 1;

        if self.1.is_some() {
            count += 1;
        }

        if self.2.is_some() {
            count += 1;
        }

        if self.3.is_some() {
            count += 1;
        }

        count
    }
}

// TODO: Use these implementations also in the core lib and write unit tests
impl From<&KeyCombination> for [Option<Key>; 4] {
    fn from(value: &KeyCombination) -> Self {
        let mut keys = [None; 4];

        keys[0] = Some(value.0);
        keys[1] = value.1;
        keys[2] = value.2;
        keys[3] = value.3;

        keys
    }
}

impl From<KeyCombination> for [Option<Key>; 4] {
    fn from(value: KeyCombination) -> Self {
        From::from(&value)
    }
}

impl Hash for KeyCombination {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: [Option<Key>; 4] = self.into();
        keys.sort();
        keys.into_iter().for_each(|key| key.hash(state));
    }
}

impl PartialEq for KeyCombination {
    fn eq(&self, other: &Self) -> bool {
        if self.count_keys() != other.count_keys() {
            return false;
        }

        let self_slice: [Option<Key>; 4] = self.into();
        let other_slice: [Option<Key>; 4] = other.into();

        self_slice.iter().all(|key| other_slice.contains(key))
    }
}

impl Eq for KeyCombination {}

// TODO: Add this implementation in the core library too
impl TryFrom<&[Key]> for KeyCombination {
    type Error = ();

    fn try_from(raw_keys: &[Key]) -> Result<Self, Self::Error> {
        if raw_keys.len() > 4 {
            // TOOD: Add an actual error type in the library for this
            return Err(());
        }

        if raw_keys.is_empty() {
            // TOOD: Add an actual error type in the library for this
            return Err(());
        }

        Ok(Self(
            raw_keys[0],
            raw_keys.get(1).copied(),
            raw_keys.get(2).copied(),
            raw_keys.get(3).copied(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub action: Action,
    pub key: Key,
}

#[derive(Debug, Clone, Copy)]
pub enum ChangeEventRequest {
    None,
    Block,
    ReplaceWith(KeyCombination),
}

pub struct EventProcessor {
    switch_key: Key,
    default_combination: Option<KeyCombination>,
    mappings: collections::HashMap<KeyCombination, KeyCombination>,
    currently_pressed: Vec<Key>,
    block_events: bool,
    key_combination_executed: bool,
}

impl EventProcessor {
    // TODO: Create a From-Trait Implementation for AklConfiguration
    pub fn new(
        switch_key: Key,
        default_combination: Option<KeyCombination>,
        mappings: collections::HashMap<KeyCombination, KeyCombination>,
    ) -> Self {
        Self {
            switch_key,
            default_combination,
            mappings,
            currently_pressed: vec![],
            block_events: false,
            key_combination_executed: false,
        }
    }

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

                Debugger::write(&format!("Currently pressed: {:?}", self.currently_pressed));

                let maybe_target_combination: Result<KeyCombination, _> =
                    self.currently_pressed.as_slice().try_into();

                if let Ok(target_combination) = maybe_target_combination {
                    if let Some(replacement_combination) = self.mappings.get(&target_combination) {
                        self.key_combination_executed = true;
                        self.currently_pressed.pop();
                        return ChangeEventRequest::ReplaceWith(*replacement_combination);
                    }
                }

                ChangeEventRequest::Block
            }
            Action::Release => {
                if event.key == self.switch_key {
                    self.block_events = false;

                    if !self.key_combination_executed {
                        if let Some(combination) = self.default_combination {
                            return ChangeEventRequest::ReplaceWith(combination);
                        }
                    }

                    self.key_combination_executed = false;
                    return ChangeEventRequest::Block;
                }

                if let Ok(index) = self.currently_pressed.binary_search(&event.key) {
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
