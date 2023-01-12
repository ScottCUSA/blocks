use std::collections::HashMap;

use macroquad::{
    audio::{set_sound_volume, Sound},
    prelude::*,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::{game::RustrisGame, playfield::TranslationDirection, rustomino::RotationDirection};

// default control settings
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Left), Some(KeyCode::A)];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Right), Some(KeyCode::D)];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Up), Some(KeyCode::W)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftControl), Some(KeyCode::Z)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Down), Some(KeyCode::S)];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftShift), Some(KeyCode::C)];

// input repeat delays
const TRANSLATE_ACTION_DELAY: f64 = 0.14;
const TRANSLATE_ACTION_REPEAT_DELAY: f64 = 0.030;

const ROTATE_ACTION_DELAY: f64 = 0.14;
const ROTATE_ACTION_REPEAT_DELAY: f64 = 0.2;

// volume change amount per key press
const MUSIC_VOLUME_CHANGE: f32 = 0.025;

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

    pub fn handle_held_playing_inputs(&mut self, game: &mut RustrisGame, delta_time: f64) {
        // check each input
        for input in Controls::iter() {
            self.input_states
                .entry(input.clone()) // modify in place
                .and_modify(|e| match e {
                    InputState::Down(down_time) => {
                        // if the down time is longer than the action delay for this input
                        // change it to held
                        if let Some(action_delay) = input.action_delay_for_input() {
                            *down_time += delta_time;
                            if *down_time >= action_delay {
                                *e = InputState::Held(0.0);
                            }
                        }
                    }
                    // if the input state is held, add delta time to the held time
                    InputState::Held(held_time) => {
                        *held_time += delta_time;
                    }
                    _ => (),
                });
            if let Some(state) = self.input_states.get_mut(&input) {
                // if this state input is in a held state
                if let InputState::Held(held_time) = state {
                    // check to see if the key has been held longer than the repeat delay for
                    // the input
                    if let Some(action_repeat_delay) = input.action_repeat_delay_for_input() {
                        if *held_time >= action_repeat_delay {
                            // if it is then call the input function
                            *state = InputState::Held(0.0);
                            match input {
                                Controls::Left => game.translate(TranslationDirection::Left),
                                Controls::Right => game.translate(TranslationDirection::Right),
                                Controls::RotateCW => game.rotate(RotationDirection::Cw),
                                Controls::RotateCCW => game.rotate(RotationDirection::Ccw),
                                Controls::SoftDrop => game.soft_drop(),
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn handle_playing_inputs(&mut self, game: &mut RustrisGame) {
        if is_key_pressed(KeyCode::Escape) {
            self.clear_inputs();
            game.pause();
        }

        for (input, keys) in &self.input_map.clone() {
            for key in keys.iter().flatten() {
                if is_key_pressed(*key) {
                    self.input_states
                        .entry(input.clone())
                        .and_modify(|e| *e = InputState::Down(0.0));
                    match input {
                        Controls::Left => game.translate(TranslationDirection::Left),
                        Controls::Right => game.translate(TranslationDirection::Right),
                        Controls::RotateCW => game.rotate(RotationDirection::Cw),
                        Controls::RotateCCW => game.rotate(RotationDirection::Ccw),
                        Controls::SoftDrop => game.soft_drop(),
                        Controls::HardDrop => {
                            self.clear_inputs();
                            game.hard_drop();
                        }
                        Controls::Hold => {
                            self.clear_inputs();
                            game.hold();
                        }
                    }
                } else if is_key_released(*key) {
                    self.input_states
                        .entry(input.clone())
                        .and_modify(|e| *e = InputState::Up);
                }
            }
        }
    }

    pub fn handle_paused_inputs(&mut self, game: &mut RustrisGame) {
        if is_key_pressed(KeyCode::Escape) {
            self.clear_inputs();
            game.resume();
        }
    }

    pub fn handle_game_over_inputs(&mut self, game: &mut RustrisGame) {
        if is_key_pressed(KeyCode::Enter) {
            self.clear_inputs();
            game.play_again();
        }
    }

    pub fn handle_menu_inputs(&mut self, game: &mut RustrisGame) {
        if is_key_pressed(KeyCode::Enter) {
            game.resume();
        }
    }
}

pub fn handle_global_controls(background_music: &Sound, music_volume: &mut f32) {
    // allow control of the game volume
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        *music_volume -= MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        set_sound_volume(*background_music, *music_volume);
        log::debug!("volume decrease {}", music_volume);
    }

    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        *music_volume += MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        set_sound_volume(*background_music, *music_volume);
        log::debug!("volume increase {}", music_volume);
    }
}
