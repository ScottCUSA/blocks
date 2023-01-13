use crate::{
    controls::ControlStates,
    playfield::{RustrisPlayfield, TranslationDirection, PLAYFIELD_SIZE},
    rustomino::*,
};
use ::rand::{seq::SliceRandom, SeedableRng};
use std::f64::consts::E;
use strum::IntoEnumIterator;

use macroquad::prelude::*;

const GRAVITY_NUMERATOR: f64 = 1.0; // how
const GRAVITY_FACTOR: f64 = 0.1; // slow or increase gravity factor
const LINES_PER_LEVEL: usize = 10; // used to increase or decrease how quickly the game progresses
const LOCKDOWN_MAX_TIME: f64 = 0.5; // lockdown delay is 0.5 seconds
const LOCKDOWN_MAX_RESETS: u32 = 15; // maximum number of times the lockdown timer can be reset
const STARTING_LEVEL: usize = 0;

// const DEBUG_RNG_SEED: u64 = 123456789; // for debugging RNG
// const DELAY_TO_LOCK: f64 = 0.5; // how long to wait before locking a block which cannot move down
// const MAX_DELAY_RESETS: i32 = 10; // how many times to reset the delay

const SINGLE_LINE_SCORE: usize = 100;
const DOUBLE_LINE_SCORE: usize = 300;
const TRIPLE_LINE_SCORE: usize = 500;
const RUSTRIS_SCORE: usize = 800;

pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

/// returns the delay for the level in fractional seconds
fn gravity_delay(level: usize) -> f64 {
    let gravity_delay =
        ((GRAVITY_NUMERATOR / (level as f64 + 0.001)).log(E) * GRAVITY_FACTOR + 0.3).max(0.001);
    log::info!("new gravity_delay {}", gravity_delay);
    gravity_delay
}

pub struct RustrisGame {
    pub playfield: RustrisPlayfield,
    pub next_rustomino: Option<Rustomino>,
    pub held_rustomino: Option<Rustomino>,
    pub state: GameState,
    pub level: usize,
    pub score: usize,
    rustomino_bag: Vec<RustominoType>, // contains the next rustomino types, shuffled
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    gravity_delay: f64, // time between gravity ticks
    total_lines_cleared: usize,
    last_update: f64, // when did the last game update occur
    hold_used: bool,  // if user has held a rustomino, resets on lock
    lockdown_resets: u32,
    current_rustomino_state: CurrentRustominoState, // state of current rustomino, falling/lockdown
}

pub enum CurrentRustominoState {
    Falling { time: f64 },
    Lockdown { time: f64 },
}

impl RustrisGame {
    pub fn new(playfield: RustrisPlayfield) -> Self {
        RustrisGame {
            playfield,
            next_rustomino: None,
            held_rustomino: None,
            state: GameState::Menu, // Start the game at the menu screen
            score: 0,
            level: STARTING_LEVEL,
            rustomino_bag: Vec::new(),
            rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(),
            gravity_delay: gravity_delay(0),
            total_lines_cleared: 0,
            last_update: get_time(),
            hold_used: false,
            lockdown_resets: 0,
            current_rustomino_state: CurrentRustominoState::Falling { time: 0.0 },
        }
        .init()
    }

    fn init(mut self) -> Self {
        log::info!("Initializing RustrisGame");
        self.get_next_rustomino();
        self
    }

    pub fn update(&mut self, controls: &mut ControlStates) {
        let now = get_time();
        let delta_time = now - self.last_update;

        // handle the game states
        match self.state {
            GameState::Menu => {
                // handle the user's inputs
                controls.handle_menu_inputs(self);
            }
            GameState::Playing => {
                // check to see if the playfield is ready for the next rustomino
                if self.playfield.ready_for_next() {
                    // take the next rustomino
                    if let Some(current_rustomino) = self.next_rustomino.take() {
                        // we used next_rustomino so we need to replace it
                        self.get_next_rustomino();
                        // new rustomino state is always falling
                        self.current_rustomino_state = CurrentRustominoState::Falling { time: 0.0 };
                        // add the next rustomino to the playfield
                        if !self.playfield.set_current_rustomino(current_rustomino) {
                            // game over if it can't be placed without a collision
                            self.game_over();
                            return;
                        }
                    } else {
                        log::debug!("next_rustomomino was None when it shouldn't have been.");
                        self.get_next_rustomino();
                        return;
                    }
                }

                controls.handle_playing_inputs(self);
                controls.handle_held_playing_inputs(self, delta_time);

                // handle the current rustomino
                match self.current_rustomino_state {
                    CurrentRustominoState::Falling {
                        time: mut gravity_time_accum,
                    } => {
                        // if the block is falling accumulate the time delta
                        gravity_time_accum += delta_time;
                        // if the accumulated time exceeds the current gravity delay
                        // apply gravity
                        if gravity_time_accum >= self.gravity_delay {
                            // check to see if the playfield's current rustomino can fall
                            if self.playfield.can_fall() {
                                log::debug!("playfield:\n{}", self.playfield);
                                // apply gravity if it can
                                self.playfield.apply_gravity();
                                // reset the accumulated time
                                self.current_rustomino_state =
                                    CurrentRustominoState::Falling { time: 0.0 };
                            } else {
                                // the block can't move down so it's state becomes "Lockdown"
                                // If this block has been in Lockdown state before
                                if self.lockdown_resets > 0 {
                                    log::debug!(
                                        "incrementing lockdown resets: {}",
                                        self.lockdown_resets
                                    );
                                    // hitting the deck again causes a lockdown reset
                                    self.lockdown_resets += 1;
                                }
                                self.current_rustomino_state =
                                    CurrentRustominoState::Lockdown { time: 0. };
                            }
                        } else {
                            // no condition has been met to lock the block so
                            // update the accumulated gravity time
                            self.current_rustomino_state = CurrentRustominoState::Falling {
                                time: gravity_time_accum,
                            };
                        }
                    }
                    CurrentRustominoState::Lockdown {
                        time: mut lockdown_time,
                    } => {
                        // if the block is currently in a lockdown state
                        // we need to add the time delta to the lockdown time
                        lockdown_time += delta_time;
                        // check to see if the block can fall again,
                        if self.playfield.can_fall() {
                            // if so set the state back to falling
                            self.current_rustomino_state =
                                CurrentRustominoState::Falling { time: 0.0 };
                        } else if lockdown_time >= LOCKDOWN_MAX_TIME {
                            // if the current lockdown time has exceed the maximum
                            // lock the block
                            self.lock("lockdown time expired");
                        } else if self.lockdown_resets >= LOCKDOWN_MAX_RESETS {
                            // if the user has exceeded the maximum number of resets
                            // lock the block
                            self.lock("max lockdown resets reached");
                        } else {
                            // no condition has been met to lock the block so continue
                            // accumulating lockdown time
                            self.current_rustomino_state = CurrentRustominoState::Lockdown {
                                time: lockdown_time,
                            };
                        }
                    }
                }
            }
            GameState::Paused => {
                controls.handle_paused_inputs(self);
            }
            GameState::GameOver => {
                controls.handle_game_over_inputs(self);
            }
        }
        self.last_update = now;
    }

    fn increase_game_level(&mut self) {
        self.level += 1;
        log::info!("increasing game level to {}", self.level);
        self.gravity_delay = gravity_delay(self.level);
    }

    fn get_next_rustomino(&mut self) {
        // this can be called even if next_rustomino is some
        // in this case do nothing
        if self.next_rustomino.is_some() {
            return;
        }

        // if we've used all of the rustomino's fill the bag
        self.fill_rustomino_bag();

        if let Some(next_type) = self.rustomino_bag.pop() {
            log::debug!("next rustomino: {:?}", next_type);
            self.next_rustomino = Some(Rustomino::new(next_type));
        }
    }

    // add one of each rustomino type to bag
    // then shuffle the bag
    fn fill_rustomino_bag(&mut self) {
        if !self.rustomino_bag.is_empty() {
            log::debug!("rustomino bag: {:?}", self.rustomino_bag);
            return;
        }
        self.rustomino_bag
            .append(&mut RustominoType::iter().collect());
        self.rustomino_bag.shuffle(&mut self.rng);
        log::debug!("filled rustomino bag: {:?}", self.rustomino_bag);
    }

    fn lock(&mut self, reason: &str) {
        if let Some(rustomino) = &self.playfield.current_rustomino {
            log::info!(
                "locking rustomnio for {reason}; type: {:?} blocks: {:?}",
                rustomino.rustomino_type,
                rustomino.playfield_slots()
            );
            // check for full out of bounds lockout (game over)
            if fully_out_of_bounds(&rustomino.playfield_slots()) {
                self.game_over();
                return;
            }
        }
        self.hold_used = false;
        self.playfield.lock_rustomino();

        self.lockdown_resets = 0;
        self.handle_completed_lines();
    }

    // incrementing the number of lockdown resets
    // reseting the lockdown time to 0
    fn reset_lockdown(&mut self) {
        if let CurrentRustominoState::Lockdown { time: _ } = self.current_rustomino_state {
            self.lockdown_resets += 1;
            log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
            self.current_rustomino_state = CurrentRustominoState::Lockdown { time: 0. }
        };
    }

    pub fn translate(&mut self, direction: TranslationDirection) {
        if self.playfield.translate_current(direction) {
            self.reset_lockdown();
        }
    }

    pub fn rotate(&mut self, direction: RotationDirection) {
        if self.playfield.rotate_current(direction) {
            self.reset_lockdown();
        }
    }

    pub fn soft_drop(&mut self) {
        if !self.playfield.translate_current(TranslationDirection::Down) {
            self.lock("soft drop");
        }
        self.current_rustomino_state = CurrentRustominoState::Falling { time: 0. };
    }

    pub fn hard_drop(&mut self) {
        self.playfield.hard_drop();
        self.lock("hard drop");

        self.current_rustomino_state = CurrentRustominoState::Falling { time: 0. };
    }

    // Hold action. Hold a rustomino for later use.
    // If a rustomino has not yet been held, the current rustomino is held,
    // and the next rustomino is added to the playfield
    // If a rustomino is already held, this rustomino is added to the playfield,
    // and the current rustomino is held
    // The player can't use the hold action again until the current rustomino is locked
    pub fn hold(&mut self) {
        // check to see if the player has used the hold action
        // and they haven't yet locked the rustomino they took
        if self.hold_used {
            return;
        }
        // check to see if there is a held rustomino
        let rustomino = if self.held_rustomino.is_some() {
            // take the held_rustomino
            self.held_rustomino.take().unwrap()
        } else {
            // if not we take the next rustomino
            self.next_rustomino.take().unwrap()
        };

        // if we used next_rustomino we need to replace it
        self.get_next_rustomino();

        // take current_rustomino and make it the hold_rustomino
        self.held_rustomino = self.playfield.take_current();
        self.playfield.set_current_rustomino(rustomino);

        // prevent the player from taking the hold action again
        // until the next rustomino is locked
        self.hold_used = true;
    }

    fn game_over(&mut self) {
        log::info!("Game Over! Score: {}", self.score);
        self.state = GameState::GameOver;
    }

    fn handle_completed_lines(&mut self) {
        let completed_lines = self.playfield.clear_completed_lines();
        if completed_lines.is_empty() {
            return;
        }
        self.total_lines_cleared += completed_lines.len();
        log::info!("number of completed lines: {}", self.total_lines_cleared);
        self.score_completed_lines(completed_lines.len());

        // increase the game level every LINES_PER_LEVEL
        if self.total_lines_cleared >= (self.level + 1) * LINES_PER_LEVEL {
            self.increase_game_level();
        }
    }

    fn score_completed_lines(&mut self, num_completed_lines: usize) {
        // Single line 100xlevel
        // Double line 300xlevel
        // Triple line 500xlevel
        // Rustris (4 lines) 800xlevel
        let score = match num_completed_lines {
            1 => SINGLE_LINE_SCORE,
            2 => DOUBLE_LINE_SCORE,
            3 => TRIPLE_LINE_SCORE,
            4 => RUSTRIS_SCORE,
            _ => {
                panic!("impossibru")
            }
        } * (self.level + 1);
        self.score += score;
        log::info!(
            "scored! game_level: {} score: {} total score: {}",
            self.level,
            score,
            self.score
        )
    }

    pub fn pause(&mut self) {
        log::info!("game paused");
        self.state = GameState::Paused;
    }

    pub fn resume(&mut self) {
        log::info!("game resumed");
        self.state = GameState::Playing;
    }

    pub fn play_again(&mut self) {
        log::info!("starting new game");
        self.state = GameState::Playing;
        self.playfield = RustrisPlayfield::new();
        self.next_rustomino = None;
        self.held_rustomino = None;
        self.state = GameState::Playing;
        self.current_rustomino_state = CurrentRustominoState::Falling { time: 0.0 };
        self.score = 0;
        self.level = STARTING_LEVEL;
        self.hold_used = false;
        self.rustomino_bag = Vec::new();
        self.gravity_delay = gravity_delay(1);
        self.total_lines_cleared = 0;
        self.last_update = get_time();
        self.get_next_rustomino();
    }
}

fn fully_out_of_bounds(&slots: &[IVec2; 4]) -> bool {
    // check for out of bounds lockout
    // if any slot is not out of bounds return false
    for slot in slots {
        if slot[1] < PLAYFIELD_SIZE[1] {
            return false;
        }
    }
    return true;
}