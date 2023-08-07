use ggez::input::keyboard::KeyCode;
use std::{collections::HashMap, time};
use strum::{EnumIter, IntoEnumIterator};

// default control settings
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Left), Some(KeyCode::A)];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Right), Some(KeyCode::D)];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Up), Some(KeyCode::W)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LControl), Some(KeyCode::Z)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Down), Some(KeyCode::S)];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LShift), Some(KeyCode::C)];

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
    Down(time::Instant),
    Held(time::Instant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Control {
    Left,
    Right,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    Hold,
}

impl Control {
    pub fn action_delay(&self) -> Option<f64> {
        match self {
            Control::Left | Control::Right => Some(TRANSLATE_ACTION_DELAY),
            Control::SoftDrop => Some(SOFT_DROP_ACTION_DELAY),
            _ => None,
        }
    }
    pub fn action_repeat_delay(&self) -> Option<f64> {
        match self {
            Control::Left | Control::Right => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Control::SoftDrop => Some(SOFT_DROP_ACTION_REPEAT_DELAY),
            _ => None,
        }
    }
    pub fn default_keys(&self) -> [Option<KeyCode>; 2] {
        match self {
            Control::Left => LEFT_KEYS,
            Control::Right => RIGHT_KEYS,
            Control::RotateCW => ROTATE_CW_KEYS,
            Control::RotateCCW => ROTATE_CCW_KEYS,
            Control::SoftDrop => SOFT_DROP_KEYS,
            Control::HardDrop => HARD_DROP_KEYS,
            Control::Hold => HOLD_KEYS,
        }
    }
}

pub struct GameControls {
    pub input_map: HashMap<Control, [Option<KeyCode>; 2]>,
    pub key_map: HashMap<KeyCode, Control>,
    pub input_states: HashMap<Control, InputState>,
}

impl Default for GameControls {
    fn default() -> Self {
        Self {
            input_map: { Control::iter().map(|i| (i, i.default_keys())).collect() },
            key_map: {
                LEFT_KEYS
                    .iter()
                    .flatten()
                    .map(|e| (*e, Control::Left))
                    .chain(RIGHT_KEYS.iter().flatten().map(|e| (*e, Control::Right)))
                    .chain(
                        ROTATE_CW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Control::RotateCW)),
                    )
                    .chain(
                        ROTATE_CCW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Control::RotateCCW)),
                    )
                    .chain(
                        SOFT_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Control::SoftDrop)),
                    )
                    .chain(
                        HARD_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Control::HardDrop)),
                    )
                    .chain(HOLD_KEYS.iter().flatten().map(|e| (*e, Control::Hold)))
                    .collect::<HashMap<KeyCode, Control>>()
            },
            input_states: {
                Control::iter()
                    .map(|e| (e, InputState::default()))
                    .collect::<HashMap<Control, InputState>>()
            },
        }
    }
}

impl GameControls {
    pub fn clear_inputs(&mut self) {
        for input in Control::iter() {
            self.input_states
                .entry(input)
                .and_modify(|e| *e = InputState::Up);
        }
    }

    pub fn set_pressed(&mut self, keycode: Option<KeyCode>) {
        for (key, input) in self.key_map.iter() {
            if keycode == Some(*key) {
                self.input_states
                    .entry(*input)
                    .and_modify(|e| *e = InputState::Down(time::Instant::now()));
            }
        }
    }

    pub fn set_released(&mut self, keycode: Option<KeyCode>) {
        for (key, input) in self.key_map.iter() {
            if keycode == Some(*key) {
                self.input_states
                    .entry(*input)
                    .and_modify(|e| *e = InputState::Up);
            }
        }
    }
}
