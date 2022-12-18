use std::collections::VecDeque;

use crate::rustominos::*;
use crate::rustris_board::RustrisBoard;
use piston_window::Key;
use rand::distributions::{Distribution, Standard};
use rand::SeedableRng;

const DEBUG_SEED: u64 = 123456789;
const SIZE_NEXT_RUSTOMINOS: usize = 20;
const STARTING_GRAVITY_DELAY: f64 = 0.5; // in MS?
const GRAVITY_INCREASE_EXPONENT: u32 = 2;
const GRAVITY_INCREASE_FACTOR: f64 = 0.2; // used to increase or slowdown rate of exponential growth
const BLOCKS_PER_LEVEL: usize = 10;

pub struct RustrisOptions {
    starting_gravity_delay: f64,
}

impl RustrisOptions {
    pub fn new() -> Self {
        RustrisOptions {
            starting_gravity_delay: STARTING_GRAVITY_DELAY,
        }
    }
    pub fn gravity_delay(&mut self, level: usize) -> f64 {
        (self.starting_gravity_delay
            - ((level.pow(GRAVITY_INCREASE_EXPONENT) as f64) * GRAVITY_INCREASE_FACTOR))
            .max(0.001)
    }
}

pub struct RustrisController {
    board: RustrisBoard,
    next_rustominos: VecDeque<RustominoType>,
    next_rustomino: Option<Rustomino>,
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    options: RustrisOptions,
    game_level: usize,
    gravity_time_accum: f64,
    gravity_delay: f64,
}

impl RustrisController {
    pub fn new(board: RustrisBoard) -> Self {
        RustrisController {
            board,
            next_rustominos: VecDeque::new(),
            rng: rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(DEBUG_SEED),
            // rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(), // USE THIS WHEN NOT TESTING
            next_rustomino: None,
            options: RustrisOptions::new(),
            game_level: 1,
            gravity_time_accum: 0.0,
            gravity_delay: 0.0, // set with self.options.gravity_delay(game_level)
        }
    }

    pub fn init(mut self) -> Self {
        log::info!("Initializing RustrisController");
        self.fill_next_rustominos(SIZE_NEXT_RUSTOMINOS);
        self.next_rustomino = Some(self.get_next_rustomino());
        self.gravity_delay = self.options.gravity_delay(self.game_level);
        return self;
    }

    fn increase_game_level(&mut self) {
        log::info!(
            "increasing game level from {} to {}",
            self.game_level,
            self.game_level + 1,
        );
        self.game_level += 1;
        self.gravity_delay = self.options.gravity_delay(self.game_level);
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
    }

    pub fn update(&mut self, delta_time: f64) {
        // place rustomino on the board in it's starting location

        if self.board.check_need_next() {
            // if we used next_rustomino we need to replace it
            if self.next_rustomino.is_none() {
                self.next_rustomino = Some(self.get_next_rustomino());
            }
            // unwrap should be OK here because we are making sure it's not None immediately before this
            let current_rustomino = self.next_rustomino.take().unwrap();
            self.board.add_new_rustomino(current_rustomino);
        }

        // Apply "gravity" to move the rustomino down the board, or if it can't move lock it
        self.gravity_time_accum = self.gravity_time_accum + delta_time;
        if self.gravity_time_accum >= self.gravity_delay {
            self.gravity_time_accum = 0.0;
            self.board.gravity_tick();
            println!("{}", self.board);
        }

        // increase the game level every BLOCKS_PER_LEVEL
        if self.board.num_locked_rustominos() / self.game_level == BLOCKS_PER_LEVEL {
            self.increase_game_level();
        }
    }
}
