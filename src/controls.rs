use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::winit::keyboard::PhysicalKey;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};

// input repeat delays, before the first repeat and between repeats
const TRANSLATE_REPEAT: (f64, f64) = (0.3, 0.025);
const SOFT_DROP_REPEAT: (f64, f64) = (0.2, 0.03);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputState {
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
    /// repeat delays while held, None acts once per press
    pub fn repeat(&self) -> Option<(f64, f64)> {
        match self {
            Control::Left | Control::Right => Some(TRANSLATE_REPEAT),
            Control::SoftDrop => Some(SOFT_DROP_REPEAT),
            _ => None,
        }
    }

    pub fn default_keys(&self) -> &'static [KeyCode] {
        match self {
            Control::Left => &[KeyCode::ArrowLeft, KeyCode::KeyA],
            Control::Right => &[KeyCode::ArrowRight, KeyCode::KeyD],
            Control::RotateCW => &[KeyCode::ArrowUp, KeyCode::KeyW],
            Control::RotateCCW => &[KeyCode::ControlLeft, KeyCode::KeyZ],
            Control::SoftDrop => &[KeyCode::ArrowDown, KeyCode::KeyS],
            Control::HardDrop => &[KeyCode::Space],
            Control::Hold => &[KeyCode::ShiftLeft, KeyCode::KeyC],
        }
    }
}

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
                        .iter()
                        .map(move |&key| (key, control))
                })
                .collect(),
            input_states: Control::iter().map(|c| (c, InputState::Up)).collect(),
        }
    }
}

/// physical key code of a key event
pub fn keycode(input: &KeyInput) -> Option<KeyCode> {
    match input.event.physical_key {
        PhysicalKey::Code(code) => Some(code),
        PhysicalKey::Unidentified(_) => None,
    }
}

impl GameControls {
    pub fn clear_inputs(&mut self) {
        self.input_states
            .values_mut()
            .for_each(|state| *state = InputState::Up);
    }

    pub fn set_pressed(&mut self, keycode: Option<KeyCode>) {
        self.set(keycode, InputState::Down(0.0));
    }

    pub fn set_released(&mut self, keycode: Option<KeyCode>) {
        self.set(keycode, InputState::Up);
    }

    fn set(&mut self, keycode: Option<KeyCode>, state: InputState) {
        if let Some(&control) = keycode.and_then(|key| self.key_map.get(&key)) {
            self.input_states.insert(control, state);
        }
    }
}
