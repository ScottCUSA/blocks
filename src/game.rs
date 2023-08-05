use crate::{
    controls::{self, ControlStates, Controls, InputState},
    draw,
    playfield::{RustrisPlayfield, TranslationDirection, PLAYFIELD_SIZE},
    rustomino::{Rotation, Rustomino, RustominoBag, RustominoState},
    util::variants_equal,
};
use macroquad::{
    audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams, Sound},
    prelude::*,
};
use std::f64::consts::E;
use strum::IntoEnumIterator;

// GAMEPLAY CONSTANTS
const GRAVITY_NUMERATOR: f64 = 1.0;
const GRAVITY_FACTOR: f64 = 0.1; // used to slow or increase gravity factor
const STARTING_LEVEL: usize = 0;
const LINES_PER_LEVEL: usize = 10; // number of lines that need to be cleared before level advances
const LOCKDOWN_DELAY: f64 = 0.5; // how long to wait before locking block (Tetris Guideline)
const LOCKDOWN_MAX_RESETS: u32 = 15; // maximum number of times the lockdown timer can be reset (Tetris Guideline)

// SCORING CONSTANTS
const SINGLE_LINE_SCORE: usize = 100;
const DOUBLE_LINE_SCORE: usize = 300;
const TRIPLE_LINE_SCORE: usize = 500;
const RUSTRIS_SCORE: usize = 800;

// ASSET CONSTANTS
const ASSETS_FOLDER: &str = "assets";
const MUSIC_VOL: f32 = 0.1;
const MUSIC_VOLUME_CHANGE: f32 = 0.025;

pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

pub struct RustrisGame {
    pub playfield: RustrisPlayfield,
    pub next_rustomino: Option<Rustomino>,
    pub held_rustomino: Option<Rustomino>,
    pub state: GameState,
    pub level: usize,
    pub score: usize,
    rustomino_bag: RustominoBag,
    gravity_delay: f64, // time between gravity ticks
    total_lines_cleared: usize,
    hold_used: bool, // if user has held a rustomino, resets on lock
    lockdown_resets: u32,
}

impl RustrisGame {
    fn new(playfield: RustrisPlayfield) -> Self {
        RustrisGame {
            playfield,
            next_rustomino: None,
            held_rustomino: None,
            state: GameState::Menu, // Start the game at the menu screen
            level: STARTING_LEVEL,
            score: 0,
            rustomino_bag: RustominoBag::new(),
            gravity_delay: gravity_delay(0),
            total_lines_cleared: 0,
            hold_used: false,
            lockdown_resets: 0,
        }
    }

    fn playing_update(&mut self, delta_time: f64) {
        let Some(current_state) = self.playfield.get_active_state() else {
            return;
        };
        match current_state {
            RustominoState::Falling { time } if time + delta_time >= self.gravity_delay => {
                // check to see if the block can still fall
                if self.playfield.active_can_fall() {
                    // apply gravity if it can
                    self.playfield.apply_gravity();
                    // reset the accumulated time
                    self.playfield
                        .set_active_state(RustominoState::Falling { time: 0. });
                } else {
                    // if the block can't fall, set it's state to lockdown
                    self.set_lockdown();
                }
            }
            RustominoState::Falling { time } => {
                self.playfield.set_active_state(RustominoState::Falling {
                    time: time + delta_time,
                });
            }
            RustominoState::Lockdown { time: _ }
                if self.lockdown_resets >= LOCKDOWN_MAX_RESETS
                    && !self.playfield.active_can_fall() =>
            {
                // if the user has exceeded the maximum number of resets
                // lock the block
                log::info!("maximum lockdown resets exceeded");
                self.lock();
            }
            RustominoState::Lockdown { time }
                if time + delta_time >= LOCKDOWN_DELAY && !self.playfield.active_can_fall() =>
            {
                // if the current lockdown time has exceed the maximum
                // lock the block
                log::info!("lockdown time expired");
                self.lock();
            }
            RustominoState::Lockdown { time } => {
                // accumulate lockdown time
                self.playfield.set_active_state(RustominoState::Lockdown {
                    time: time + delta_time,
                });
            }
        }
    }

    fn set_lockdown(&mut self) {
        // check if this block has been in Lockdown state before
        if self.lockdown_resets > 0 {
            // hitting the deck again causes a lockdown reset
            self.lockdown_resets += 1;
            log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
        }
        log::debug!("setting active rustomino state to lockdown");
        self.playfield
            .set_active_state(RustominoState::Lockdown { time: 0. });
    }

    fn get_next_rustomino(&mut self) -> Rustomino {
        // get the current next_rustomino
        // by replaceing it's value with one from the bag
        let next_rustomino = self
            .next_rustomino
            .replace(self.rustomino_bag.get_rustomino());
        // check there was a rustomino
        match next_rustomino {
            // return it if there was
            Some(rustomino) => rustomino,
            None => {
                // take the next rustomino
                let Some(rustomino) = self.next_rustomino
                    .replace(self.rustomino_bag.get_rustomino())
                else {
                    unreachable!("rustomino bag is empty");
                };
                rustomino
            }
        }
    }

    fn ready_playfield(&mut self) {
        // check to see if the playfield is ready for the next rustomino
        if self.playfield.ready_for_next() {
            log::debug!("playfield is ready for next rustomino");

            // take the next rustomino
            let active_rustomino = self.get_next_rustomino();

            // add the next rustomino to the playfield
            if !self.playfield.set_active(active_rustomino) {
                // game over if it can't be placed without a collision
                self.game_over();
            }
        }
    }

    fn translate(&mut self, direction: TranslationDirection) {
        log::debug!("translate called, direction: {:?}", direction);
        if self.playfield.translate_active(direction) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    fn rotate(&mut self, rotation: Rotation) {
        log::debug!("rotate called, direction: {:?}", rotation);
        if self.playfield.rotate_active(rotation) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    // performs a soft drop
    fn soft_drop(&mut self) {
        log::debug!("soft drop called");
        // attempt to translate the block down
        if !self.playfield.translate_active(TranslationDirection::Down) {
            // per the teris guide we shouldn't lock a block with soft drop
            let Some(state) = self.playfield.get_active_state() else {
                panic!("soft_drop called when there isn't an active state!");
            };
            // check if the block state is already in lockdown
            if !variants_equal(&state, &RustominoState::Lockdown { time: 0.0 }) {
                self.set_lockdown();
            }
            // else do nothing
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    fn hard_drop(&mut self) {
        self.playfield.hard_drop_active();
        log::info!("hard drop");
        self.lock();
        log::trace!("playfield:\n{}", self.playfield);
    }

    // Hold action. Hold a rustomino for later use.
    // If a rustomino has not yet been held, the active rustomino is held,
    // and the next rustomino is added to the playfield
    // If a rustomino is already held, this rustomino is added to the playfield,
    // and the active rustomino is held
    // The player can't use the hold action again until the active rustomino is locked
    fn hold(&mut self) {
        // check to see if the player has used the hold action
        // and they haven't yet locked the previous block they took from hold
        if self.hold_used {
            return;
        }

        // check to see if there is a held rustomino
        let next_rustomino = match self.held_rustomino.take() {
            Some(rustomino) => rustomino,      // use the held rustomino
            None => self.get_next_rustomino(), // use the next rustomino
        };

        // take active_rustomino and make it the hold_rustomino
        self.held_rustomino = self.playfield.take_active();

        // trigger game over in the unusual circumstance
        // a collision with a locked block occurs
        // when the next rustomino is added to the board
        if !self.playfield.set_active(next_rustomino.reset()) {
            log::info!("couldn't add held piece to board, collided with lock block");
            self.game_over();
        }

        // prevent the player from taking the hold action again
        // until the next rustomino is locked
        self.hold_used = true;
    }

    fn pause(&mut self) {
        log::info!("game paused");
        self.state = GameState::Paused;
    }

    fn resume(&mut self) {
        log::info!("game resumed");
        self.state = GameState::Playing;
    }

    fn game_over(&mut self) {
        log::info!("Game Over! Score: {}", self.score);
        self.state = GameState::GameOver;
    }

    fn new_game(self) -> Self {
        RustrisGame::new(RustrisPlayfield::new())
    }

    fn increase_game_level(&mut self) {
        self.level += 1;
        log::info!("increasing game level to {}", self.level);
        // get the gravity tick delay for the next level
        self.gravity_delay = gravity_delay(self.level);
    }

    fn lock(&mut self) {
        let Some(rustomino) = &self.playfield.active_rustomino else {
            return;
        };

        log::trace!("locking block: playfield:\n{}", self.playfield);
        log::trace!(
            "type: {:?} blocks: {:?}",
            rustomino.rtype,
            rustomino.playfield_slots()
        );

        // if the block we've been asked to lock is fully
        // out of bounds the game is over
        if fully_out_of_bounds(&rustomino.playfield_slots()) {
            log::debug!("block we are locking is fully out of playfield");
            self.game_over();
            return;
        }

        self.hold_used = false;
        self.playfield.lock_active();

        self.lockdown_resets = 0;
        self.handle_completed_lines();
    }

    // increment the number of lockdown resets
    // and reset the lockdown time to 0
    fn increment_lockdown_resets(&mut self) {
        let Some(active_state) = self.playfield.get_active_state() else {
            return;
        };
        // this is handled differently depending on the active rustomino's state
        match active_state {
            /*
            if the active rustomino is in a Falling state
            we only want to increment the lockdown counter
            if the current block is on the stack (can't fall)
            and the block has previously been in locked down (lockdown_resets > 0)
            */
            RustominoState::Falling { time: _ }
                if !self.playfield.active_can_fall() && self.lockdown_resets > 0 =>
            {
                log::debug!("block can't fall setting rustomino state back to lockdown");
                // set the state back to lockdown
                self.playfield
                    .set_active_state(RustominoState::Lockdown { time: 0. });
                self.lockdown_resets += 1;
                log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
            }
            RustominoState::Lockdown { time: _ } => {
                self.lockdown_resets += 1;
                log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
                // if the block can fall again it needs to continue falling
                if self.playfield.active_can_fall() {
                    // if so set the state back to falling
                    log::debug!("block can fall setting rustomino state back to falling");
                    self.playfield
                        .set_active_state(RustominoState::Falling { time: 0. });
                } else {
                    log::debug!("resetting lockdown timer");
                    self.playfield
                        .set_active_state(RustominoState::Lockdown { time: 0. });
                }
            }
            _ => {}
        }
    }

    fn handle_completed_lines(&mut self) {
        let cleared_lines = self.playfield.clear_completed_lines();
        if cleared_lines.is_empty() {
            return;
        }

        let num_lines_cleared = cleared_lines.len();

        // score the completed lines and append it to the total score
        let score = score_cleared_lines(num_lines_cleared, self.level);
        self.score += score;
        log::info!(
            "scored! game_level: {} score: {} lines cleared: {}",
            self.level,
            score,
            num_lines_cleared
        );

        // track the total number of lines cleared
        self.total_lines_cleared += num_lines_cleared;
        log::info!(
            "total number of cleared lines: {}",
            self.total_lines_cleared
        );

        // increase the game level every LINES_PER_LEVEL
        if self.total_lines_cleared >= (self.level + 1) * LINES_PER_LEVEL {
            self.increase_game_level();
        }
    }
}

fn score_cleared_lines(num_lines: usize, level: usize) -> usize {
    // Single line 100xlevel
    // Double line 300xlevel
    // Triple line 500xlevel
    // Rustris (4 lines) 800xlevel
    (level + 1)
        * match num_lines {
            1 => SINGLE_LINE_SCORE,
            2 => DOUBLE_LINE_SCORE,
            3 => TRIPLE_LINE_SCORE,
            4 => RUSTRIS_SCORE,
            _ => {
                panic!("impossible number of lines cleared")
            }
        }
}

// checks to see if ALL of the slots in the provided
// slots array are above the playfield
fn fully_out_of_bounds(&slots: &[IVec2; 4]) -> bool {
    for slot in slots {
        if slot[1] < PLAYFIELD_SIZE[1] {
            return false;
        }
    }
    true
}

/// calculate the gravity delay for the provided level
/// returns fractional seconds
fn gravity_delay(level: usize) -> f64 {
    let gravity_delay =
        ((GRAVITY_NUMERATOR / (level as f64 + 0.001)).log(E) * GRAVITY_FACTOR + 0.3).max(0.001);
    log::info!("new gravity_delay {}", gravity_delay);
    gravity_delay
}

// run the game
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
    let font = load_ttf_font(&font_path.to_string_lossy()).await.ok();

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
        &background_music,
        PlaySoundParams {
            looped: true,
            volume: music_volume,
        },
    );

    let mut last_update = get_time();

    loop {
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
        draw::draw(&game, font.as_ref());

        last_update = get_time();

        next_frame().await;
    }
}

// returns a closure which handles the provided
// control for the game
fn control_handler<'a>(control: &'a Controls, game: &'a mut RustrisGame) -> Box<dyn FnMut() + 'a> {
    match *control {
        Controls::Left => Box::new(|| game.translate(TranslationDirection::Left)),
        Controls::Right => Box::new(|| game.translate(TranslationDirection::Right)),
        Controls::RotateCW => Box::new(|| game.rotate(Rotation::Cw)),
        Controls::RotateCCW => Box::new(|| game.rotate(Rotation::Ccw)),
        Controls::SoftDrop => Box::new(|| game.soft_drop()),
        Controls::HardDrop => Box::new(|| game.hard_drop()),
        Controls::Hold => Box::new(|| game.hold()),
    }
}

fn handle_global_inputs(background_music: &Sound, music_volume: &mut f32) {
    // volume down
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        *music_volume -= MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        set_sound_volume(background_music, *music_volume);
        log::debug!("volume decrease {}", music_volume);
    }
    // volume up
    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        *music_volume += MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        set_sound_volume(background_music, *music_volume);
        log::debug!("volume increase {}", music_volume);
    }
}

fn handle_playing_inputs(control_states: &mut ControlStates, game: &mut RustrisGame) {
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
                control_handler(input, game)();
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

// Some of the games controls allow repeating their actions
// when the user holds their inputs
// This handles updating the state of these inputs
// as well as calling game functions when appropriate
fn handle_held_playing_inputs(
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
                    // check to see if the key is repeatable
                    // and if the down time is longer than the action delay for this input
                    if let Some(action_delay) = control.action_delay() {
                        *down_time += delta_time;
                        if *down_time >= action_delay {
                            *e = InputState::Held(0.);
                            control_handler(&control, game)();
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
            // if this input is in a held state
            if let InputState::Held(held_time) = state {
                // check if held was just set
                if *held_time == 0. {
                    // call the game control handler function
                    control_handler(&control, game)();
                }
                // check to see if the key is repeatable
                // and if the key has been held longer than the repeat delay for the input
                if let Some(action_repeat_delay) = control.action_repeat_delay() {
                    if *held_time >= action_repeat_delay {
                        // reset the held state time
                        *state = InputState::Held(0.);
                        // call the game control handler function
                        control_handler(&control, game)();
                    }
                }
            }
        }
    }
}
