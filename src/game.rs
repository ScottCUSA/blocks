use crate::{
    playfield::{RustrisPlayfield, TranslationDirection, PLAYFIELD_SIZE},
    rustomino::{RotationDirection, Rustomino, RustominoBag, RustominoState},
};
use macroquad::prelude::*;
use std::f64::consts::E;

// GAMEPLAY CONSTANTS
const GRAVITY_NUMERATOR: f64 = 1.0;
const GRAVITY_FACTOR: f64 = 0.1; // used to slow or increase gravity factor
const STARTING_LEVEL: usize = 0;
const LINES_PER_LEVEL: usize = 10; // number of lines that need to be cleared before level advances
const LOCKDOWN_MAX_TIME: f64 = 0.5; // how long to wait before locking block
const LOCKDOWN_MAX_RESETS: u32 = 15; // maximum number of times the lockdown timer can be reset

// SCORING CONSTANTS
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
    pub fn new(playfield: RustrisPlayfield) -> Self {
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

    fn ensure_next_rustomino(&mut self) {
        // make sure next_rustomino is available
        if self.next_rustomino.is_none() {
            self.next_rustomino = Some(self.rustomino_bag.get_next_rustomino());
        }
    }

    pub fn ready_playfield(&mut self) {
        // make sure next_rustomino is available
        self.ensure_next_rustomino();
        // check to see if the playfield is ready for the next rustomino
        if self.playfield.ready_for_next() {
            log::debug!("playfield is ready for next rustomino");
            // take the next rustomino
            let current_rustomino = self.next_rustomino.take().unwrap();
            // this makes sure next_rustomino is set
            self.ensure_next_rustomino();
            // add the next rustomino to the playfield
            if !self.playfield.play_rustomino(current_rustomino) {
                // game over if it can't be placed without a collision
                self.game_over();
            }
        }
    }

    pub fn playing_update(&mut self, delta_time: f64) {
        let Some(current_state) = self.playfield.get_active_state() else {
            // no active state
            return;
        };
        match current_state {
            RustominoState::Falling { time } if time + delta_time >= self.gravity_delay => {
                // check to see if the block can still fall
                if self.playfield.can_fall() {
                    // apply gravity if it can
                    self.playfield.apply_gravity();
                    // reset the accumulated time
                    self.playfield
                        .set_active_state(RustominoState::Falling { time: 0. });
                } else {
                    // the block can't move down so it's state becomes "Lockdown"
                    // If this block has been in Lockdown state before
                    if self.lockdown_resets > 0 {
                        // hitting the deck again causes a lockdown reset
                        self.lockdown_resets += 1;
                        log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
                    }
                    log::debug!("setting current rustomino state to lockdown");

                    self.playfield
                        .set_active_state(RustominoState::Lockdown { time: 0. });
                }
            }
            RustominoState::Falling { time } => {
                self.playfield.set_active_state(RustominoState::Falling {
                    time: time + delta_time,
                });
            }
            RustominoState::Lockdown { time: _ }
                if self.lockdown_resets >= LOCKDOWN_MAX_RESETS && !self.playfield.can_fall() =>
            {
                // if the user has exceeded the maximum number of resets
                // lock the block
                self.lock("max lockdown resets reached");
            }
            RustominoState::Lockdown { time }
                if time + delta_time >= LOCKDOWN_MAX_TIME && !self.playfield.can_fall() =>
            {
                // if the current lockdown time has exceed the maximum
                // lock the block
                self.lock("lockdown time expired");
            }
            RustominoState::Lockdown { time } => {
                // accumulate lockdown time
                self.playfield.set_active_state(RustominoState::Lockdown {
                    time: time + delta_time,
                });
            }
        }
    }

    fn increase_game_level(&mut self) {
        self.level += 1;
        log::info!("increasing game level to {}", self.level);
        self.gravity_delay = gravity_delay(self.level);
    }

    fn lock(&mut self, reason: &str) {
        log::trace!("locking block: playfield:\n{}", self.playfield);
        if let Some(rustomino) = &self.playfield.active_rustomino {
            log::debug!("locking rustomnio for {reason}");
            log::trace!(
                "type: {:?} blocks: {:?}",
                rustomino.rtype,
                rustomino.playfield_slots()
            );
            // check for full out of bounds lockout (game over)
            if fully_out_of_bounds(&rustomino.playfield_slots()) {
                log::debug!("block we are locking is fully out of playfield");
                self.game_over();
                return;
            }
        }

        self.hold_used = false;
        self.playfield.lock_rustomino();

        self.lockdown_resets = 0;
        self.handle_completed_lines();
    }

    // increment the number of lockdown resets
    // and reset the lockdown time to 0
    fn increment_lockdown_resets(&mut self) {
        let Some(active_state) = self.playfield.get_active_state() else {
            // no active state
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
                if !self.playfield.can_fall() && self.lockdown_resets > 0 =>
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
                if self.playfield.can_fall() {
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

    pub fn translate(&mut self, direction: TranslationDirection) {
        log::debug!("translate called, direction: {:?}", direction);
        if self.playfield.translate_current(direction) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    pub fn rotate(&mut self, direction: RotationDirection) {
        log::debug!("rotate called, direction: {:?}", direction);
        if self.playfield.rotate_current(direction) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    pub fn soft_drop(&mut self) {
        log::debug!("soft drop called");
        if !self.playfield.translate_current(TranslationDirection::Down) {
            self.lock("soft drop");
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    pub fn hard_drop(&mut self) {
        log::debug!("hard drop called");

        self.playfield.hard_drop();
        self.lock("hard drop");
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
            self.next_rustomino
                .take()
                .unwrap_or(self.rustomino_bag.get_next_rustomino())
        };

        // if we used next_rustomino we need to replace it
        self.ensure_next_rustomino();

        // take current_rustomino and make it the hold_rustomino
        self.held_rustomino = self.playfield.take_active();

        if !self.playfield.play_rustomino(rustomino.reset()) {
            log::info!("on hold, replacement piece couldn't be added to the board");
            self.game_over();
        }

        // prevent the player from taking the hold action again
        // until the next rustomino is locked
        self.hold_used = true;
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

    fn game_over(&mut self) {
        log::info!("Game Over! Score: {}", self.score);
        self.state = GameState::GameOver;
    }

    pub fn new_game(self) -> Self {
        RustrisGame::new(RustrisPlayfield::new())
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
