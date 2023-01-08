use crate::{
    board::{RustrisBoard, TranslationDirection},
    controls::ControlStates,
    rustomino::*,
};
use ::rand::{seq::SliceRandom, SeedableRng};
use std::f64::consts::E;
use strum::IntoEnumIterator;

use macroquad::prelude::*;

const GRAVITY_NUMERATOR: f64 = 1.0; // how
const GRAVITY_FACTOR: f64 = 4.0; // slow or increase gravity factor
const LINES_PER_LEVEL: usize = 10; // how many blocks between levels (should this be score based?)

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
        (GRAVITY_NUMERATOR / (((level + 1) as f64).log(E) * GRAVITY_FACTOR)).max(0.001);
    log::info!("new gravity_delay {}", gravity_delay);
    gravity_delay
}

pub struct RustrisGame {
    pub board: RustrisBoard,
    pub next_rustomino: Option<Rustomino>,
    pub held_rustomino: Option<Rustomino>,
    pub game_state: GameState,
    pub score: usize,
    pub game_level: usize,
    rustomino_bag: Vec<RustominoType>,
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    gravity_time_accum: f64,
    gravity_delay: f64,
    completed_lines: usize,
    last_update: f64,
    hold_used: bool,
}

impl RustrisGame {
    pub fn new(board: RustrisBoard) -> Self {
        RustrisGame {
            board,
            next_rustomino: None,
            held_rustomino: None,
            game_state: GameState::Menu, // GameState::Menu,
            rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(),
            score: 0,
            game_level: 1,
            hold_used: false,
            rustomino_bag: Vec::new(),
            gravity_time_accum: 0.0,
            gravity_delay: gravity_delay(1),
            completed_lines: 0,
            last_update: get_time(),
        }
        .init()
    }

    fn init(mut self) -> Self {
        log::info!("Initializing RustrisGame");
        self.get_next_rustomino();
        self
    }

    fn increase_game_level(&mut self) {
        self.game_level += 1;
        log::info!("increasing game level to {}", self.game_level);
        self.gravity_delay = gravity_delay(self.game_level);
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

    fn gravity_tick(&mut self) {
        // check to see if the board's current rustomino can fall
        let movable = self.board.can_fall();

        log::debug!("board:\n{}", self.board);
        log::debug!("gravity tick, rustomino movable: {movable}");

        if movable {
            self.board.apply_gravity();
        } else {
            self.lock("gravity tick");
        }
    }

    fn lock(&mut self, reason: &str) {
        if let Some(rustomino) = &self.board.current_rustomino {
            log::info!(
                "locking rustomnio for {reason}; type: {:?} blocks: {:?}",
                rustomino.rustomino_type,
                rustomino.board_slots()
            );
        }
        self.hold_used = false;
        self.board.lock_rustomino();

        self.handle_completed_lines();
    }

    pub fn translate(&mut self, direction: TranslationDirection) {
        self.board.translate_current(direction);
    }

    pub fn rotate(&mut self, direction: RotationDirection) {
        self.board.rotate_current(direction);
    }

    pub fn soft_drop(&mut self) {
        if !self.board.translate_current(TranslationDirection::Down) {
            self.lock("soft drop");
        }
        self.gravity_time_accum = 0.0;
    }

    pub fn hard_drop(&mut self) {
        self.board.hard_drop();
        self.lock("hard drop");
        self.gravity_time_accum = 0.0;
    }

    // Hold action. Hold a rustomino for later use.
    // If a rustomino has not yet been held, the current rustomino is held,
    // and the next rustomino is added to the board
    // If a rustomino is already held, this rustomino is added to the board,
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
        self.held_rustomino = self.board.take_current();
        self.board.set_current_rustomino(rustomino);

        // prevent the player from taking the hold action again
        // until the next rustomino is locked
        self.hold_used = true;
    }

    fn game_over(&mut self) {
        log::info!("Game Over! Score: {}", self.score);
        self.game_state = GameState::GameOver;
    }

    fn handle_completed_lines(&mut self) {
        let completed_lines = self.board.clear_completed_lines();
        if completed_lines.is_empty() {
            return;
        }
        self.completed_lines += completed_lines.len();
        log::info!("number of completed lines: {}", self.completed_lines);
        self.score_completed_lines(completed_lines.len());
        // increase the game level every LINES_PER_LEVEL
        if self.completed_lines >= self.game_level * LINES_PER_LEVEL {
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
        } * self.game_level;
        self.score += score;
        log::info!(
            "scored! game_level: {} score: {} total score: {}",
            self.game_level,
            score,
            self.score
        )
    }

    pub fn pause(&mut self) {
        log::info!("game paused");
        self.game_state = GameState::Paused;
    }

    pub fn resume(&mut self) {
        log::info!("game resumed");
        self.game_state = GameState::Playing;
    }

    pub fn play_again(&mut self) {
        log::info!("starting new game");
        self.game_state = GameState::Playing;
        self.board = RustrisBoard::new();
        self.next_rustomino = None;
        self.held_rustomino = None;
        self.game_state = GameState::Playing;
        self.score = 0;
        self.game_level = 1;
        self.hold_used = false;
        self.rustomino_bag = Vec::new();
        self.gravity_time_accum = 0.0;
        self.gravity_delay = gravity_delay(1);
        self.completed_lines = 0;
        self.last_update = get_time();
        self.get_next_rustomino();
    }

    pub fn update(&mut self, controls: &mut ControlStates) {
        let now = get_time();
        let delta_time = now - self.last_update;

        match self.game_state {
            GameState::Menu => {
                controls.handle_menu_inputs(self);
            }
            GameState::Playing => {
                // check board ready for the next rustomino
                if self.board.ready_for_next() {
                    // TODO: move this whole block to a fn
                    // take the next rustomino
                    // unwrap should be safe here
                    let current_rustomino = self.next_rustomino.take().unwrap();
                    // we used next_rustomino so we need to replace it
                    self.get_next_rustomino();
                    // add the next rustomino to the board
                    // game over if it can't be placed without a collision
                    if !self.board.set_current_rustomino(current_rustomino) {
                        self.game_over();
                    }
                }

                controls.handle_playing_inputs(self);
                controls.handle_held_playing_inputs(self, delta_time);

                // Apply "gravity" to move the current rustomino down the board
                // or if it can't move lock it
                self.gravity_time_accum += delta_time;
                if self.gravity_time_accum >= self.gravity_delay {
                    self.gravity_time_accum = 0.0;
                    self.gravity_tick();
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
}
