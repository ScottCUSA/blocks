use crate::board::{RustrisBoard, TranslationDirection};
use crate::rustomino::*;
use crate::view::RustrisView;
use opengl_graphics::GlGraphics;
use piston_window::{
    Button, Key, PistonWindow, PressEvent, ReleaseEvent, RenderEvent, ResizeEvent, UpdateEvent,
};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::HashMap;
use std::f64::consts::E;
use strum::{EnumIter, IntoEnumIterator};

const GRAVITY_NUMERATOR: f64 = 1.0; // how
const GRAVITY_FACTOR: f64 = 2.0; // slow or increase gravity factor
const LINES_PER_LEVEL: usize = 10; // how many blocks between levels (should this be score based?)

// const DEBUG_RNG_SEED: u64 = 123456789; // for debugging RNG
// const DELAY_TO_LOCK: f64 = 0.5; // how long to wait before locking a block which cannot move down
// const MAX_DELAY_RESETS: i32 = 10; // how many times to reset the delay

const SINGLE_LINE_SCORE: usize = 100;
const DOUBLE_LINE_SCORE: usize = 300;
const TRIPLE_LINE_SCORE: usize = 500;
const RUSTRIS_SCORE: usize = 800;

const TRANSLATE_ACTION_DELAY: f64 = 0.14;
const TRANSLATE_ACTION_REPEAT_DELAY: f64 = 0.030;

const ROTATE_ACTION_DELAY: f64 = 0.14;
const ROTATE_ACTION_REPEAT_DELAY: f64 = 0.2;

// default input settings
const LEFT_KEYS: [Option<Key>; 2] = [Some(Key::Left), None];
const RIGHT_KEYS: [Option<Key>; 2] = [Some(Key::Right), None];
const ROTATE_CW_KEYS: [Option<Key>; 2] = [Some(Key::Up), Some(Key::X)];
const ROTATE_CCW_KEYS: [Option<Key>; 2] = [Some(Key::LCtrl), Some(Key::Z)];
const SOFT_DROP_KEYS: [Option<Key>; 2] = [Some(Key::Down), None];
const HARD_DROP_KEYS: [Option<Key>; 2] = [Some(Key::Space), None];
const HOLD_KEYS: [Option<Key>; 2] = [Some(Key::LShift), Some(Key::C)];

pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

// enum PlayingState {
//     Normal,
//     DelayToLock { current_delay: f64, num_resets: i32 },
// }

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyState {
    Up,
    Down(f64),
    Held(f64),
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState::Up
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
enum Inputs {
    Left,
    Right,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    Hold,
}

impl Inputs {
    // fn key_for_input(&self) -> [Option<Key>; 2] {
    //     match self {
    //         Inputs::Left => LEFT_KEYS,
    //         Inputs::Right => RIGHT_KEYS,
    //         Inputs::RotateCW => ROTATE_CW_KEYS,
    //         Inputs::RotateCCW => ROTATE_CCW_KEYS,
    //         Inputs::SoftDrop => SOFT_DROP_KEYS,
    //         Inputs::HardDrop => HARD_DROP_KEYS,
    //         Inputs::Hold => HOLD_KEYS,
    //     }
    // }
    fn action_delay_for_input(&self) -> Option<f64> {
        match self {
            Inputs::Left => Some(TRANSLATE_ACTION_DELAY),
            Inputs::Right => Some(TRANSLATE_ACTION_DELAY),
            Inputs::SoftDrop => Some(TRANSLATE_ACTION_DELAY),
            Inputs::RotateCW => Some(ROTATE_ACTION_DELAY),
            Inputs::RotateCCW => Some(ROTATE_ACTION_DELAY),
            _ => None,
        }
    }
    fn action_repeat_delay_for_input(&self) -> Option<f64> {
        match self {
            Inputs::Left => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Inputs::Right => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Inputs::SoftDrop => Some(TRANSLATE_ACTION_REPEAT_DELAY),
            Inputs::RotateCW => Some(ROTATE_ACTION_REPEAT_DELAY),
            Inputs::RotateCCW => Some(ROTATE_ACTION_REPEAT_DELAY),
            _ => None,
        }
    }
}

pub struct InputController {
    // input_keys: HashMap<Inputs, [Option<Key>; 2]>,
    input_key_map: HashMap<Key, Inputs>,
    input_states: HashMap<Inputs, KeyState>,
}

impl Default for InputController {
    fn default() -> Self {
        Self {
            // input_keys: HashMap::from([
            //     (Inputs::Left, LEFT_KEYS),
            //     (Inputs::Right, RIGHT_KEYS),
            //     (Inputs::RotateCW, ROTATE_CW_KEYS),
            //     (Inputs::RotateCCW, ROTATE_CCW_KEYS),
            //     (Inputs::SoftDrop, SOFT_DROP_KEYS),
            //     (Inputs::HardDrop, HARD_DROP_KEYS),
            //     (Inputs::Hold, HOLD_KEYS),
            // ]),
            input_key_map: {
                LEFT_KEYS
                    .iter()
                    .flatten()
                    .map(|e| (*e, Inputs::Left))
                    .chain(RIGHT_KEYS.iter().flatten().map(|e| (*e, Inputs::Right)))
                    .chain(
                        ROTATE_CW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Inputs::RotateCW)),
                    )
                    .chain(
                        ROTATE_CCW_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Inputs::RotateCCW)),
                    )
                    .chain(
                        SOFT_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Inputs::SoftDrop)),
                    )
                    .chain(
                        HARD_DROP_KEYS
                            .iter()
                            .flatten()
                            .map(|e| (*e, Inputs::HardDrop)),
                    )
                    .chain(HOLD_KEYS.iter().flatten().map(|e| (*e, Inputs::Hold)))
                    .collect::<HashMap<Key, Inputs>>()
            },
            input_states: {
                Inputs::iter()
                    .map(|e| (e, KeyState::default()))
                    .collect::<HashMap<Inputs, KeyState>>()
            },
        }
    }
}

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

pub struct RustrisController {
    pub board: RustrisBoard,
    pub next_rustomino: Option<Rustomino>,
    pub hold_rustomino: Option<Rustomino>,
    pub game_state: GameState,
    pub score: usize,
    pub game_level: usize,
    hold_set: bool,
    input_controller: InputController,
    rustomino_bag: Vec<RustominoType>,
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    gravity_time_accum: f64,
    gravity_delay: f64,
    completed_lines: usize,
}

impl RustrisController {
    pub fn new(board: RustrisBoard) -> Self {
        RustrisController {
            board,
            next_rustomino: None,
            hold_rustomino: None,
            game_state: GameState::Playing, // GameState::Menu,
            score: 0,
            game_level: 1,
            hold_set: false,
            // rng: rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(DEBUG_RNG_SEED), // FOR DEBUGING
            input_controller: InputController::default(),
            // input_controller: InputController::from (),
            rustomino_bag: Vec::new(),
            rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(),
            gravity_time_accum: 0.0,
            gravity_delay: 0.0, // self.options.gravity_delay(game_level)
            completed_lines: 0,
        }
    }

    pub fn init(mut self) -> Self {
        log::info!("Initializing RustrisController");
        self.fill_rustomino_bag();
        self.set_next_rustomino();
        self.gravity_delay = gravity_delay(self.game_level);
        self
    }

    pub fn run(
        &mut self,
        window: &mut PistonWindow,
        opengl: &mut GlGraphics,
        view: &mut RustrisView,
    ) {
        while let Some(event) = window.next() {
            if let Some(Button::Keyboard(key)) = event.press_args() {
                self.key_pressed(key);
            }
            if let Some(Button::Keyboard(key)) = event.release_args() {
                self.key_released(key);
            }
            if let Some(args) = event.resize_args() {
                view.resize(args);
            }
            if let Some(args) = event.render_args() {
                opengl.draw(args.viewport(), |c, g| view.draw(self, &c, g));
            }
            event.update(|arg| {
                self.update(arg.dt);
            });
        }
    }

    pub fn key_pressed(&mut self, key: Key) {
        // allow the user to rotate the rustomino with the left and right arrows
        // allow the user to fast drop the rustomino with the down arrow key
        log::debug!("key pressed: {:?}", key);

        match self.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                if let Some(input) = self.input_controller.input_key_map.get(&key) {
                    self.input_controller
                        .input_states
                        .entry(*input)
                        .and_modify(|e| *e = KeyState::Down(0.0));
                    match input {
                        Inputs::Left => self.translate(TranslationDirection::Left),
                        Inputs::Right => self.translate(TranslationDirection::Right),
                        Inputs::RotateCW => self.rotate(RotationDirection::Cw),
                        Inputs::RotateCCW => self.rotate(RotationDirection::Ccw),
                        Inputs::SoftDrop => self.soft_drop(),
                        Inputs::HardDrop => self.hard_drop(),
                        Inputs::Hold => self.hold(),
                    }
                }
            }
            GameState::GameOver => todo!(),
        }
    }

    pub fn key_released(&mut self, key: Key) {
        // allow the user to rotate the rustomino with the left and right arrows
        // allow the user to fast drop the rustomino with the down arrow key
        log::debug!("key released: {:?}", key);
        match self.game_state {
            GameState::Playing => {
                if let Some(input) = self.input_controller.input_key_map.get(&key) {
                    self.input_controller
                        .input_states
                        .entry(*input)
                        .and_modify(|e| *e = KeyState::Up)
                        .or_insert(KeyState::default());
                }
            }
            GameState::Menu => todo!(),
            GameState::GameOver => todo!(),
        }
    }

    fn handle_inputs(&mut self, delta_time: f64) {
        // check each input
        for input in Inputs::iter() {
            self.input_controller
                .input_states
                .entry(input)
                .and_modify(|e| match e {
                    KeyState::Down(down_time) => {
                        if let Some(action_delay) = input.action_delay_for_input() {
                            *down_time += delta_time;
                            if *down_time >= action_delay {
                                *e = KeyState::Held(0.0);
                            }
                        }
                    }
                    KeyState::Held(held_time) => {
                        *held_time += delta_time;
                    }
                    _ => (),
                });
            if let Some(state) = self.input_controller.input_states.get_mut(&input) {
                match state {
                    KeyState::Held(held_time) => {
                        if let Some(action_repeat_delay) = input.action_repeat_delay_for_input() {
                            if *held_time >= action_repeat_delay {
                                *state = KeyState::Held(0.0);
                                match input {
                                    Inputs::Left => self.translate(TranslationDirection::Left),
                                    Inputs::Right => self.translate(TranslationDirection::Right),
                                    Inputs::RotateCW => self.rotate(RotationDirection::Cw),
                                    Inputs::RotateCCW => self.rotate(RotationDirection::Ccw),
                                    Inputs::SoftDrop => self.soft_drop(),
                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        match self.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                // check board ready for the next rustomino
                if self.board.ready_for_next() {
                    // TODO: move this whole block to a fn
                    // take the next rustomino
                    // unwrap should be safe here
                    let current_rustomino = self.next_rustomino.take().unwrap();
                    // we used next_rustomino so we need to replace it
                    self.set_next_rustomino();
                    // add the next rustomino to the board
                    // game over if it can't be placed without a collision
                    if !self.board.add_new_rustomino(current_rustomino) {
                        self.game_over();
                    }
                }
                self.handle_inputs(delta_time);
                // Apply "gravity" to move the current rustomino down the board
                // or if it can't move lock it
                self.gravity_time_accum += delta_time;
                if self.gravity_time_accum >= self.gravity_delay {
                    self.gravity_time_accum = 0.0;
                    self.gravity_tick();
                }

                // increase the game level every LINES_PER_LEVEL
                if self.completed_lines > self.game_level * LINES_PER_LEVEL {
                    self.increase_game_level();
                }
            }
            GameState::GameOver => todo!(),
        }
    }

    fn increase_game_level(&mut self) {
        self.game_level += 1;
        log::info!("increasing game level to {}", self.game_level);
        self.gravity_delay = gravity_delay(self.game_level);
    }

    fn set_next_rustomino(&mut self) {
        // we don't need to set the next rustomino
        if self.next_rustomino.is_some() {
            return;
        }
        if self.rustomino_bag.is_empty() {
            self.fill_rustomino_bag();
        }
        // unwrap is OK because we are making sure next_rustomino's is never empty
        let next_rustomino = Rustomino::new(self.rustomino_bag.pop().unwrap());
        log::debug!("Next Rustomino:\n{next_rustomino}");

        self.next_rustomino = Some(next_rustomino);
    }

    // add one of each rustomino type to bag
    // then shuffle the bag
    fn fill_rustomino_bag(&mut self) {
        self.rustomino_bag
            .append(&mut RustominoType::iter().collect());
        self.rustomino_bag.shuffle(&mut self.rng);
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
        self.hold_set = false;
        self.board.lock_rustomino();

        self.handle_completed_lines();
    }

    fn translate(&mut self, direction: TranslationDirection) {
        self.board.translate_current(direction);
    }

    fn rotate(&mut self, direction: RotationDirection) {
        self.board.rotate_current(direction);
    }

    fn soft_drop(&mut self) {
        if !self.board.translate_current(TranslationDirection::Down) {
            self.lock("soft drop");
        }
        self.gravity_time_accum = 0.0;
    }

    fn hard_drop(&mut self) {
        self.board.hard_drop();
        self.lock("hard drop");
        self.gravity_time_accum = 0.0;
    }

    // if there is no current hold piece
    // we need to hold the current piece
    // then move the next piece onto the board
    //
    // if there is a held piece
    // we need to swap the current and held piece
    // then place the new current piece onto the board
    // player can only do this once until the next block is locked
    fn hold(&mut self) {
        // can only hold once
        if self.hold_set || self.board.current_rustomino.is_none() {
            return;
        }
        let temp_rustomino = if self.hold_rustomino.is_some() {
            // take the hold_rustomino
            self.hold_rustomino.take().unwrap()
        } else {
            // take the hold_rustomino
            self.next_rustomino.take().unwrap()
        };

        // if we used next_rustomino so we need to replace it
        self.set_next_rustomino();
        // reset current_rustomino and make it the hold_rustomino
        self.hold_rustomino = Some(self.board.current_rustomino.take().unwrap().reset());
        self.board.add_new_rustomino(temp_rustomino);

        self.hold_set = true;
    }

    fn game_over(&mut self) {
        log::info!("Game Over!");
        self.game_state = GameState::GameOver;
    }

    fn handle_completed_lines(&mut self) {
        let completed_lines = self.board.clear_completed_lines();
        if completed_lines.is_empty() {
            return;
        }
        self.completed_lines += completed_lines.len();
        self.score_completed_lines(completed_lines);
    }

    fn score_completed_lines(&mut self, completed_lines: Vec<usize>) {
        // Single line 100xlevel
        // Double line 300xlevel
        // Triple line 500xlevel
        // Rustris (4 lines) 800xlevel
        let score = match completed_lines.len() {
            1 => {
                log::info!("scored! single line");
                SINGLE_LINE_SCORE
            }
            2 => {
                log::info!("scored! double line");
                DOUBLE_LINE_SCORE
            }
            3 => {
                log::info!("scored! triple line");
                TRIPLE_LINE_SCORE
            }
            4 => {
                log::info!("scored! rustris");
                RUSTRIS_SCORE
            }
            _ => {
                panic!("shouldn't be able to score more than 4 l ines")
            }
        };
        let score = score * self.game_level;
        self.score += score;
        log::info!(
            "scored! game_level: {} score: {} total score: {}",
            self.game_level,
            score,
            self.score
        )
    }
}
