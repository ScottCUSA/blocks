use macroquad::prelude::*;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};

// default control settings
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Left), Some(KeyCode::A)];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Right), Some(KeyCode::D)];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Up), Some(KeyCode::W)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftControl), Some(KeyCode::Z)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Down), Some(KeyCode::S)];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftShift), Some(KeyCode::C)];

// input repeat delays
const TRANSLATE_ACTION_DELAY: f64 = 0.3;
const TRANSLATE_ACTION_REPEAT_DELAY: f64 = 0.025;
const SOFT_DROP_ACTION_DELAY: f64 = 0.2;
const SOFT_DROP_ACTION_REPEAT_DELAY: f64 = 0.03;

// TODO: implement saving and loading inputs from file

#[derive(Debug, Clone, PartialEq, Default)]
pub enum InputState {
    #[default]
    Up,
    Down(f64),
    Held(f64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Controls {
    Left,
    Right,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    Hold,
}

impl Controls {
    pub fn action_delay(&self) -> Option<f64> {
        match self {
            Controls::Left | Controls::Right => Some(TRANSLATE_ACTION_DELAY),
            Controls::SoftDrop => Some(SOFT_DROP_ACTION_DELAY),
            _ => None,
        }
    }
    pub fn action_repeat_delay(&self) -> Option<f64> {
        match self {
            Controls::Left | Controls::Right => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Controls::SoftDrop => Some(SOFT_DROP_ACTION_REPEAT_DELAY),
            _ => None,
        }
    }
    pub fn default_keys(&self) -> [Option<KeyCode>; 2] {
        match self {
            Controls::Left => LEFT_KEYS,
            Controls::Right => RIGHT_KEYS,
            Controls::RotateCW => ROTATE_CW_KEYS,
            Controls::RotateCCW => ROTATE_CCW_KEYS,
            Controls::SoftDrop => SOFT_DROP_KEYS,
            Controls::HardDrop => HARD_DROP_KEYS,
            Controls::Hold => HOLD_KEYS,
        }
    }
}

pub struct ControlStates {
    pub input_map: HashMap<Controls, [Option<KeyCode>; 2]>,
    pub key_map: HashMap<KeyCode, Controls>,
    pub input_states: HashMap<Controls, InputState>,
}

impl Default for ControlStates {
    fn default() -> Self {
        Self {
            input_map: {
                Controls::iter()
                    .map(|i| (i.clone(), i.default_keys()))
                    .collect()
            },
            key_map: {
                LEFT_KEYS
                    .iter()
                    .flatten()
                    .map(|e| (*e, Controls::Left))
                    .chain(RIGHT_KEYS.iter().flatten().map(|e| (*e, Controls::Right)))
                    .chain(
                        ROTATE_CW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Controls::RotateCW)),
                    )
                    .chain(
                        ROTATE_CCW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Controls::RotateCCW)),
                    )
                    .chain(
                        SOFT_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Controls::SoftDrop)),
                    )
                    .chain(
                        HARD_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Controls::HardDrop)),
                    )
                    .chain(HOLD_KEYS.iter().flatten().map(|e| (*e, Controls::Hold)))
                    .collect::<HashMap<KeyCode, Controls>>()
            },
            input_states: {
                Controls::iter()
                    .map(|e| (e, InputState::default()))
                    .collect::<HashMap<Controls, InputState>>()
            },
        }
    }
}

impl ControlStates {
    pub fn clear_inputs(&mut self) {
        for input in Controls::iter() {
            self.input_states
                .entry(input.clone())
                .and_modify(|e| *e = InputState::Up);
        }
    }
}
