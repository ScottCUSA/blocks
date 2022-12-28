use std::collections::VecDeque;

use crate::board::RustrisBoard;
use crate::rustomino::*;
use log::info;
use piston_window::Key;
use rand::distributions::{Distribution, Standard};
use rand::SeedableRng;

const DEBUG_RNG_SEED: u64 = 123456789; // for RNG
const SIZE_NEXT_RUSTOMINOS: usize = 20; // How many rustomino types to generate ahead of time
const GRAVITY_NUMERATOR: f64 = 1.0; // how
const GRAVITY_FACTOR: f64 = 4.0; // slow or increase gravity factor
const BLOCKS_PER_LEVEL: usize = 1;
const E: f64 = 2.7182818284;

// an attempt at a customizable logaritmically decreasing delay
//                 GRAVITY_NUMERATOR
// delay =  --------------------------------
//          (ln(level + 1) * GRAVITY_FACTOR)
fn gravity_delay(level: usize) -> f64 {
    let gravity_delay =
        (GRAVITY_NUMERATOR / (((level + 1) as f64).log(E) * GRAVITY_FACTOR)).max(0.001);
    log::info!("new gravity_delay {}", gravity_delay);
    gravity_delay
}

pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

pub struct RustrisController {
    pub board: RustrisBoard,
    next_rustominos: VecDeque<RustominoType>,
    next_rustomino: Option<Rustomino>,
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    game_level: usize,
    gravity_time_accum: f64,
    gravity_delay: f64,
    pub game_state: GameState,
    left_pressed: bool,
    right_pressed: bool,
    down_pressed: bool,
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
            gravity_delay: 0.0, // self.options.gravity_delay(game_level)
            game_state: GameState::Playing, // GameState::Menu,
            left_pressed: false,
            right_pressed: false,
            down_pressed: false,
        }
    }

    pub fn init(mut self) -> Self {
        log::info!("Initializing RustrisController");
        self.fill_next_rustominos(SIZE_NEXT_RUSTOMINOS);
        self.next_rustomino = Some(self.get_next_rustomino());
        self.gravity_delay = gravity_delay(self.game_level);
        return self;
    }

    fn increase_game_level(&mut self) {
        self.game_level += 1;
        log::info!("increasing game level to {}", self.game_level);
        self.gravity_delay = gravity_delay(self.game_level);
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
        match self.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                match key {
                    Key::Left => {
                        // pressed move left
                        self.left_pressed = true;
                        self.board.translate(TranslationDirection::Left);
                    }
                    Key::Right => {
                        // pressed move right
                        self.right_pressed = true;
                        self.board.translate(TranslationDirection::Right);
                    }
                    Key::Up | Key::X => {
                        // pressed rotate CW
                        // self.board.rotate_rustomino(RotationDirection::CW);
                    }
                    Key::LCtrl | Key::Z => {
                        // pressed rotate CCW
                        // self.board.rotate_rustomino(RotationDirection::CCW);
                    }
                    Key::Down => {
                        self.down_pressed = true;
                        // pressed soft drop
                    }
                    Key::Space => {
                        self.board.drop();
                        // pressed drop hard
                    }
                    _ => {}
                }
            }
            GameState::GameOver => todo!(),
        }
    }

    pub fn key_released(&mut self, key: Key) {
        // allow the user to rotate the rustomino with the left and right arrows
        // allow the user to fast drop the rustomino with the down arrow key
        log::info!("key released: {:?}", key);
        match self.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                match key {
                    Key::Left => {
                        // released move left
                        self.left_pressed = false;
                    }
                    Key::Right => {
                        // released move right
                        self.right_pressed = false;
                    }
                    Key::Down => {
                        // released soft drop
                        self.down_pressed = false;
                    }
                    _ => {}
                }
            }
            GameState::GameOver => todo!(),
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        match self.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                if self.board.check_need_next() {
                    // if we used next_rustomino we need to replace it
                    if self.next_rustomino.is_none() {
                        self.next_rustomino = Some(self.get_next_rustomino());
                    }
                    // unwrap should be OK here
                    // we are making sure it's not not None immediately before this
                    let current_rustomino = self.next_rustomino.take().unwrap();
                    // add the next rustomino to the board (move)
                    if !self.board.add_new_rustomino(current_rustomino) {
                        self.game_over();
                    }
                }
                // Apply "gravity" to move the rustomino down the board, or if it can't move lock it
                self.gravity_time_accum = self.gravity_time_accum + delta_time;
                if self.gravity_time_accum >= self.gravity_delay {
                    self.gravity_time_accum = 0.0;
                    self.board.gravity_tick();
                    log::debug!("delta_time:\n{}", delta_time);
                }

                // increase the game level every BLOCKS_PER_LEVEL
                if self.board.num_locked_rustominos() / self.game_level == BLOCKS_PER_LEVEL {
                    self.increase_game_level();
                }
            }
            GameState::GameOver => todo!(),
        }
    }

    fn game_over(&mut self) {
        info!("Game Over!");
        self.game_state = GameState::GameOver;
    }
}
