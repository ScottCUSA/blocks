use std::collections::VecDeque;

use crate::rustominos::*;
use crate::rustris_board::RustrisBoard;
use piston_window::Key;
use rand::distributions::{Distribution, Standard};
use rand::SeedableRng;

const DEBUG_RNG_SEED: u64 = 123456789; // for RNG
const SIZE_NEXT_RUSTOMINOS: usize = 20; // How many rustomino types to generate ahead of time
const GRAVITY_NUMERATOR: f64 = 2.0; // how
const GRAVITY_FACTOR: f64 = 4.0; // slow or increase gravity factor
const BLOCKS_PER_LEVEL: usize = 1;
const E: f64 = 2.7182818284;
pub struct RustrisOptions {}

impl RustrisOptions {
    // an attempt at a customizable logaritmically decreasing delay
    //                 GRAVITY_NUMERATOR
    // delay =  --------------------------------
    //          (ln(level + 1) * GRAVITY_FACTOR)
    pub fn gravity_delay(level: usize) -> f64 {
        (GRAVITY_NUMERATOR / (((level + 1) as f64).log(E) * GRAVITY_FACTOR)).max(0.001)
    }
}

pub struct RustrisController {
    board: RustrisBoard,
    next_rustominos: VecDeque<RustominoType>,
    next_rustomino: Option<Rustomino>,
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    game_level: usize,
    gravity_time_accum: f64,
    gravity_delay: f64,
}

impl RustrisController {
    pub fn new(board: RustrisBoard) -> Self {
        RustrisController {
            board,
            next_rustominos: VecDeque::new(),
            rng: rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(DEBUG_RNG_SEED),
            // rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(), // USE THIS WHEN NOT TESTING
            next_rustomino: None,
            game_level: 1,
            gravity_time_accum: 0.0,
            gravity_delay: 0.0, // set with self.options.gravity_delay(game_level)
        }
    }

    pub fn init(mut self) -> Self {
        log::info!("Initializing RustrisController");
        self.fill_next_rustominos(SIZE_NEXT_RUSTOMINOS);
        self.next_rustomino = Some(self.get_next_rustomino());
        self.gravity_delay = RustrisOptions::gravity_delay(self.game_level);
        return self;
    }

    fn increase_game_level(&mut self) {
        log::info!(
            "increasing game level from {} to {}",
            self.game_level,
            self.game_level + 1,
        );
        self.game_level += 1;
        self.gravity_delay = RustrisOptions::gravity_delay(self.game_level);
        log::info!("new gravity_delay {}", self.gravity_delay);
    }

    fn get_next_rustomino(&mut self) -> Rustomino {
        if self.next_rustominos.len() == 0 {
            self.fill_next_rustominos(SIZE_NEXT_RUSTOMINOS);
        }
        // unwrap is OK because we are making sure next_rustomino's is never empty
        let next_rustomino = Rustomino::new(self.next_rustominos.pop_front().unwrap());
        log::debug!("Next Rustomino:\n{next_rustomino}");
        next_rustomino
    }

    fn fill_next_rustominos(&mut self, num_rustominos: usize) {
        self.next_rustominos.append(
            &mut Standard
                .sample_iter(&mut self.rng)
                .take(num_rustominos)
                .collect(),
        );
    }

    pub fn key_pressed(&mut self, key: Key) {
        // allow the user to rotate the rustomino with the left and right arrows
        // allow the user to fast drop the rustomino with the down arrow key
        log::info!("key pressed: {:?}", key);
        match key {
            Key::Left => {
                log::info!("move left");
                // self.board.translate(TranslationDirection::Left);
            }
            Key::Right => {
                log::info!("move right")
                // self.board.translate_right(TranslationDirection::Left);
            }
            Key::Up | Key::X => {
                log::info!("rotate CW");
                // self.board.rotate_rustomino(RotationDirection::CW);
            }
            Key::LCtrl | Key::Z => {
                log::info!("rotate CCW");
                // self.board.rotate_rustomino(RotationDirection::CCW);
            }
            Key::Down => {
                log::info!("drop soft")
            }
            Key::Space => {
                log::info!("drop hard")
            }
            _ => {}
        }
    }

    pub fn key_released(&mut self, key: Key) {
        // allow the user to rotate the rustomino with the left and right arrows
        // allow the user to fast drop the rustomino with the down arrow key
        log::info!("key pressed: {:?}", key);
        match key {
            Key::Left => {
                log::info!("move left");
                // self.board.translate(TranslationDirection::Left);
            }
            Key::Right => {
                log::info!("move right")
                // self.board.translate_right(TranslationDirection::Left);
            }
            Key::Up | Key::X => {
                log::info!("rotate CW");
                // self.board.rotate_rustomino(RotationDirection::CW);
            }
            Key::LCtrl | Key::Z => {
                log::info!("rotate CCW");
                // self.board.rotate_rustomino(RotationDirection::CCW);
            }
            Key::Down => {
                log::info!("drop soft")
            }
            Key::Space => {
                log::info!("drop hard")
            }
            _ => {}
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.board.check_need_next() {
            // if we used next_rustomino we need to replace it
            if self.next_rustomino.is_none() {
                self.next_rustomino = Some(self.get_next_rustomino());
            }
            // unwrap should be OK here
            // we are making sure it's not not None immediately before this
            let current_rustomino = self.next_rustomino.take().unwrap();
            // add the next rustomino to the board (move)
            self.board.add_new_rustomino(current_rustomino);
        }

        // Apply "gravity" to move the rustomino down the board, or if it can't move lock it
        self.gravity_time_accum = self.gravity_time_accum + delta_time;
        if self.gravity_time_accum >= self.gravity_delay {
            self.gravity_time_accum = 0.0;
            self.board.gravity_tick();
            log::debug!("board:\n{}", self.board);
        }

        // increase the game level every BLOCKS_PER_LEVEL
        if self.board.num_locked_rustominos() / self.game_level == BLOCKS_PER_LEVEL {
            self.increase_game_level();
        }
    }
}
