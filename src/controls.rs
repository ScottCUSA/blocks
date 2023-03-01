use ggez::input::keyboard::KeyCode;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};

use crate::game::{control_handler, RustrisGame};

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

pub struct ControlStates {
    pub input_map: HashMap<Control, ControlInputs>,
    pub input_states: HashMap<Control, InputState>,
}

impl Default for ControlStates {
    fn default() -> Self {
        Self {
            input_map: Control::iter()
                .map(|i| (i, ControlInputs::new(i.default_keys())))
                .collect(),
            input_states: {
                Control::iter()
                    .map(|e| (e, InputState::default()))
                    .collect::<HashMap<Control, InputState>>()
            },
        }
    }
}

impl ControlStates {
    pub fn clear_inputs(&mut self) {
        for input in Control::iter() {
            self.input_states
                .entry(input)
                .and_modify(|e| *e = InputState::Up);
        }
    }
}

pub fn handle_playing_inputs(control_states: &mut ControlStates, game: &mut RustrisGame) {
    // // iterate through the controls
    // for (control, inputs) in &control_states.input_map {
    //     if let Some(input) = inputs.0 {
    //         if is_key_pressed(input) {
    //             control_states
    //                 .input_states
    //                 .entry(*control)
    //                 .and_modify(|e| *e = InputState::Down(0.0));
    //             // call the game function for this input
    //             control_handler(control, game)();
    //             // ignore the other potetntial input binding for this control
    //             continue;
    //         }
    //     }
    //     if let Some(input) = inputs.1 {
    //         if is_key_pressed(input) {
    //             control_states
    //                 .input_states
    //                 .entry(*control)
    //                 .and_modify(|e| *e = InputState::Down(0.0));
    //             // call the game function for this input
    //             control_handler(control, game)();
    //         }
    //     }
    // }
}

// Some of the games controls allow repeating their actions
// when the user holds their inputs
// This handles updating the state of these inputs
// as well as calling game functions when appropriate
pub fn handle_held_playing_inputs(
    control_states: &mut ControlStates,
    game: &mut RustrisGame,
    delta_time: f64,
) {
    // // iterate through the controls
    // for control in Control::iter() {
    //     control_states
    //         .input_states
    //         .entry(control) // modify in place
    //         .and_modify(|e| match e {
    //             InputState::Down(down_time) => {
    //                 // check to see if the key is repeatable
    //                 // and if the down time is longer than the action delay for this input
    //                 if let Some(action_delay) = control.action_delay() {
    //                     *down_time += delta_time;
    //                     if *down_time >= action_delay {
    //                         *e = InputState::Held(0.);
    //                         control_handler(&control, game)();
    //                     }
    //                 }
    //             }
    //             // if the input state is held, add delta time to the held time
    //             InputState::Held(held_time) => {
    //                 *held_time += delta_time;
    //             }
    //             _ => (),
    //         });
    //     if let Some(state) = control_states.input_states.get_mut(&control) {
    //         // if this input is in a held state
    //         if let InputState::Held(held_time) = state {
    //             // check if held was just set
    //             if *held_time == 0. {
    //                 // call the game control handler function
    //                 control_handler(&control, game)();
    //             }
    //             // check to see if the key is repeatable
    //             // and if the key has been held longer than the repeat delay for the input
    //             if let Some(action_repeat_delay) = control.action_repeat_delay() {
    //                 if *held_time >= action_repeat_delay {
    //                     // reset the held state time
    //                     *state = InputState::Held(0.);
    //                     // call the game control handler function
    //                     control_handler(&control, game)();
    //                 }
    //             }
    //         }
    //     }
    // }
}

pub fn handle_global_inputs(music_volume: &mut f32) {
    // // volume down
    // if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
    //     *music_volume -= MUSIC_VOLUME_CHANGE;
    //     *music_volume = music_volume.clamp(0.0, 1.0);
    //     set_sound_volume(*background_music, *music_volume);
    //     log::debug!("volume decrease {}", music_volume);
    // }
    // // volume up
    // if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
    //     *music_volume += MUSIC_VOLUME_CHANGE;
    //     *music_volume = music_volume.clamp(0.0, 1.0);
    //     set_sound_volume(*background_music, *music_volume);
    //     log::debug!("volume increase {}", music_volume);
    // }
}
