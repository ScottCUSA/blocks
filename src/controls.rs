use std::collections::HashMap;

use macroquad::prelude::KeyCode;
use strum::{EnumIter, IntoEnumIterator};

// default control settings
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Left), None];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Right), None];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Up), Some(KeyCode::X)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftControl), Some(KeyCode::Z)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Down), None];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftShift), Some(KeyCode::C)];

// input repeat delays
const TRANSLATE_ACTION_DELAY: f64 = 0.14;
const TRANSLATE_ACTION_REPEAT_DELAY: f64 = 0.030;

const ROTATE_ACTION_DELAY: f64 = 0.14;
const ROTATE_ACTION_REPEAT_DELAY: f64 = 0.2;

#[derive(Debug, Clone, PartialEq)]
pub enum InputState {
    Up,
    Down(f64),
    Held(f64),
}

impl Default for InputState {
    fn default() -> Self {
        InputState::Up
    }
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
    pub fn action_delay_for_input(&self) -> Option<f64> {
        match self {
            Controls::Left => Some(TRANSLATE_ACTION_DELAY),
            Controls::Right => Some(TRANSLATE_ACTION_DELAY),
            Controls::SoftDrop => Some(TRANSLATE_ACTION_DELAY),
            Controls::RotateCW => Some(ROTATE_ACTION_DELAY),
            Controls::RotateCCW => Some(ROTATE_ACTION_DELAY),
            _ => None,
        }
    }
    pub fn action_repeat_delay_for_input(&self) -> Option<f64> {
        match self {
            Controls::Left => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Controls::Right => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Controls::SoftDrop => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Controls::RotateCW => Some(ROTATE_ACTION_REPEAT_DELAY),
            Controls::RotateCCW => Some(ROTATE_ACTION_REPEAT_DELAY),
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
