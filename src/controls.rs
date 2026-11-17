use bevy::input::keyboard::KeyCode;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};

// default control settings
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ArrowLeft), Some(KeyCode::KeyA)];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ArrowRight), Some(KeyCode::KeyD)];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ArrowUp), Some(KeyCode::KeyW)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ControlLeft), Some(KeyCode::KeyZ)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ArrowDown), Some(KeyCode::KeyS)];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::ShiftLeft), Some(KeyCode::KeyC)];

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
    Down(f64), // seconds held
    Held(f64), // seconds since last repeat
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

#[derive(bevy::prelude::Resource)]
pub struct GameControls {
    pub key_map: HashMap<KeyCode, Control>,
    pub input_states: HashMap<Control, InputState>,
}

impl Default for GameControls {
    fn default() -> Self {
        Self {
            key_map: Control::iter()
                .flat_map(|control| {
                    control
                        .default_keys()
                        .into_iter()
                        .flatten()
                        .map(move |key| (key, control))
                })
                .collect(),
            input_states: Control::iter()
                .map(|e| (e, InputState::default()))
                .collect(),
        }
    }
}

/// physical key code of a key event
pub fn keycode(key: KeyCode) -> Option<KeyCode> {
    match key {
        KeyCode::Unidentified(_) => None,
        code => Some(code),
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
        if let Some(&control) = keycode.and_then(|key| self.key_map.get(&key)) {
            self.input_states.insert(control, InputState::Down(0.0));
        }
    }

    pub fn set_released(&mut self, keycode: Option<KeyCode>) {
        if let Some(&control) = keycode.and_then(|key| self.key_map.get(&key)) {
            self.input_states.insert(control, InputState::Up);
        }
    }
}
