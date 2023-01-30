use crate::{
    controls::{self, ControlStates, Controls, InputState},
    game::{GameState, RustrisGame},
    playfield::{RustrisPlayfield, TranslationDirection},
    rustomino::RotationDirection,
    view,
};
use macroquad::{
    audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams, Sound},
    prelude::*,
};
use strum::IntoEnumIterator;

// ASSET CONSTANTS
const ASSETS_FOLDER: &str = "assets";
const MUSIC_VOL: f32 = 0.1;
const MUSIC_VOLUME_CHANGE: f32 = 0.025;

pub async fn run() {
    log::info!("startup: initializing Rustris;");

    // initialize the game and control states
    let mut game = RustrisGame::new(RustrisPlayfield::new());
    let mut controls = controls::ControlStates::default();

    log::info!("loading Resources");
    // find our assets path
    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder(ASSETS_FOLDER)
        .expect("unable to find assets folder");

    // load the font
    let font_path = assets_path.join("04b30.ttf");
    log::info!("loading font: {:?}", font_path);
    let font = load_ttf_font(&font_path.to_string_lossy())
        .await
        .expect("unable to load font");

    // setup two different sized fonts
    let font_20pt = TextParams {
        font,
        font_size: 20,
        ..Default::default()
    };
    let font_30pt = TextParams {
        font,
        font_size: 30,
        ..Default::default()
    };

    // load the background music
    let background_path = assets_path.join("background.ogg");
    log::info!("loading background music: {:?}", background_path);
    let background_music = load_sound(&background_path.to_string_lossy())
        .await
        .expect("unable to load background music");

    // play background music
    let mut music_volume = MUSIC_VOL;
    log::info!("playing background music at volume: {music_volume}");
    play_sound(
        background_music,
        PlaySoundParams {
            looped: true,
            volume: music_volume,
        },
    );

    let mut last_update = get_time();

    loop {
        clear_background(view::BACKGROUND_COLOR);

        // handle global controls
        handle_global_inputs(&background_music, &mut music_volume);

        let now = get_time();
        let delta_time = now - last_update;

        // handle the game states
        match game.state {
            GameState::Menu => {
                // handle the user's inputs
                if is_key_pressed(KeyCode::Enter) {
                    controls.clear_inputs();
                    game.resume();
                }
            }
            GameState::Playing => {
                // pause the game immediately
                // clear all other inputs and continue
                if is_key_pressed(KeyCode::Escape) {
                    game.pause();
                    controls.clear_inputs();
                } else {
                    game.ready_playfield();
                    handle_playing_inputs(&mut controls, &mut game);
                    handle_held_playing_inputs(&mut controls, &mut game, delta_time);
                    game.playing_update(delta_time);
                }
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Escape) {
                    controls.clear_inputs();
                    game.resume();
                }
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Enter) {
                    controls.clear_inputs();
                    game = game.new_game();
                }
            }
        }

        // draw the menus, game, overlays, etc.
        view::draw(&game, &font_20pt, &font_30pt);

        last_update = get_time();

        next_frame().await;
    }
}

// returns the game function for the provided control
pub fn control_function<'a>(
    control: &'a Controls,
    game: &'a mut RustrisGame,
) -> Box<dyn FnMut() + 'a> {
    match *control {
        Controls::Left => Box::new(|| game.translate(TranslationDirection::Left)),
        Controls::Right => Box::new(|| game.translate(TranslationDirection::Right)),
        Controls::RotateCW => Box::new(|| game.rotate(RotationDirection::Cw)),
        Controls::RotateCCW => Box::new(|| game.rotate(RotationDirection::Ccw)),
        Controls::SoftDrop => Box::new(|| game.soft_drop()),
        Controls::HardDrop => Box::new(|| game.hard_drop()),
        Controls::Hold => Box::new(|| game.hold()),
    }
}

pub fn handle_global_inputs(background_music: &Sound, music_volume: &mut f32) {
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

// Some of the games controls allow repeating their actions
// when the user holds their inputs
// This handles updating the state of these inputs
// as well as calling game functions when appropriate
pub fn handle_held_playing_inputs(
    control_states: &mut ControlStates,
    game: &mut RustrisGame,
    delta_time: f64,
) {
    // iterate through the controls
    for control in Controls::iter() {
        control_states
            .input_states
            .entry(control.clone()) // modify in place
            .and_modify(|e| match e {
                InputState::Down(down_time) => {
                    // if the down time is longer than the action delay for this input
                    // change it to held
                    if let Some(action_delay) = control.action_delay() {
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
        if let Some(state) = control_states.input_states.get_mut(&control) {
            // if this state input is in a held state
            if let InputState::Held(held_time) = state {
                // check to see if the key has been held longer than the repeat delay for
                // the input
                if let Some(action_repeat_delay) = control.action_repeat_delay() {
                    if *held_time >= action_repeat_delay {
                        // if it is then call the input function
                        *state = InputState::Held(0.0);
                        // call game function for repeatable inputs
                        if control.repeatable() {
                            control_function(&control, game)();
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_playing_inputs(control_states: &mut ControlStates, game: &mut RustrisGame) {
    // iterate through the controls
    for (input, keys) in &control_states.input_map.clone() {
        // iterate through the configured keys for the control
        for key in keys.iter().flatten() {
            if is_key_pressed(*key) {
                control_states
                    .input_states
                    .entry(input.clone())
                    .and_modify(|e| *e = InputState::Down(0.0));

                // call game function for this input
                control_function(input, game)();

                // for these controls clear all inputs and stop processing controls
                if *input == Controls::Hold || *input == Controls::HardDrop {
                    control_states.clear_inputs();
                    return;
                }
                // ignore other input bindings for this control
                break;
            } else if is_key_released(*key) {
                control_states
                    .input_states
                    .entry(input.clone())
                    .and_modify(|e| *e = InputState::Up);
            }
        }
    }
}
