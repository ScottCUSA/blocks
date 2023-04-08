use ggez::input::keyboard::{KeyCode, KeyInput};
use std::collections::HashMap;
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

const MUSIC_VOLUME_CHANGE: f32 = 0.025;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum InputState {
    #[default]
    Up,
    Down(f64),
    Held(f64),
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
    pub fn default_keys(&self) -> &[Option<KeyCode>; 2] {
        match self {
            Control::Left => &LEFT_KEYS,
            Control::Right => &RIGHT_KEYS,
            Control::RotateCW => &ROTATE_CW_KEYS,
            Control::RotateCCW => &ROTATE_CCW_KEYS,
            Control::SoftDrop => &SOFT_DROP_KEYS,
            Control::HardDrop => &HARD_DROP_KEYS,
            Control::Hold => &HOLD_KEYS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlInputs(pub Option<KeyCode>, pub Option<KeyCode>);

impl ControlInputs {
    fn new(inputs: &[Option<KeyCode>; 2]) -> ControlInputs {
        Self(inputs[0], inputs[1])
    }
}

#[derive(Clone)]
pub struct GameControls {
    pub inputs_map: HashMap<Control, ControlInputs>,
    pub states_map: HashMap<Control, InputState>,
}

impl Default for GameControls {
    fn default() -> Self {
        Self {
            inputs_map: Control::iter()
                .map(|i| (i, ControlInputs::new(i.default_keys())))
                .collect(),
            states_map: {
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
            self.states_map
                .entry(input)
                .and_modify(|e| *e = InputState::Up);
        }
    }
}

pub fn handle_global_inputs(input: &KeyInput, music_volume: &mut f32) {
    // volume down
    if input.keycode == Some(KeyCode::Minus) || input.keycode == Some(KeyCode::NumpadSubtract) {
        *music_volume -= MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        // set_sound_volume(*background_music, *music_volume);
        log::debug!("volume decrease {}", music_volume);
    }
    // volume up
    if input.keycode == Some(KeyCode::Equals) || input.keycode == Some(KeyCode::NumpadAdd) {
        *music_volume += MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        // set_sound_volume(*background_music, *music_volume);
        log::debug!("volume increase {}", music_volume);
    }
}
