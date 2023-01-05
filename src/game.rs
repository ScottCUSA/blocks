use crate::{
    board::{RustrisBoard, SlotState, TranslationDirection},
    rustomino::*,
    view::{self, ViewSettings},
};
use ::rand::seq::SliceRandom;
use ::rand::SeedableRng;
use std::collections::HashMap;
use std::f64::consts::E;
use strum::{EnumIter, IntoEnumIterator};

use macroquad::prelude::*;

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
const LEFT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Left), None];
const RIGHT_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Right), None];
const ROTATE_CW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Up), Some(KeyCode::X)];
const ROTATE_CCW_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftControl), Some(KeyCode::Z)];
const SOFT_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Down), None];
const HARD_DROP_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::Space), None];
const HOLD_KEYS: [Option<KeyCode>; 2] = [Some(KeyCode::LeftShift), Some(KeyCode::C)];

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
    input_key_map: HashMap<KeyCode, Inputs>,
    input_states: HashMap<Inputs, KeyState>,
}

impl Default for InputController {
    fn default() -> Self {
        Self {
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
                    .collect::<HashMap<KeyCode, Inputs>>()
            },
            input_states: {
                Inputs::iter()
                    .map(|e| (e, KeyState::default()))
                    .collect::<HashMap<Inputs, KeyState>>()
            },
        }
    }
}
pub enum GameState {
    Menu,
    Playing,
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
    last_update: f64,
    view_settings: ViewSettings,
}

impl RustrisGame {
    pub fn new(board: RustrisBoard) -> Self {
        RustrisGame {
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
            gravity_delay: gravity_delay(1), // self.options.gravity_delay(game_level)
            completed_lines: 0,
            last_update: get_time(),
            view_settings: ViewSettings::new([1024., 768.]),
        }
        .init()
    }

    fn init(mut self) -> Self {
        log::info!("Initializing RustrisController");
        self.fill_rustomino_bag();
        self.set_next_rustomino();
        self
    }

    pub async fn run(self) {
        scene::add_node(self);

        loop {
            next_frame().await
        }
    }

    pub fn key_pressed(&mut self, key: KeyCode) {
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

    pub fn key_released(&mut self, key: KeyCode) {
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
        self.board.translate_rustomino(direction);
    }

    fn rotate(&mut self, direction: RotationDirection) {
        self.board.rotate_rustomino(direction);
    }

    fn soft_drop(&mut self) {
        if !self.board.translate_rustomino(TranslationDirection::Down) {
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

impl scene::Node for RustrisGame {
    fn draw(node: scene::RefMut<Self>) {
        clear_background(view::BACKGROUND_COLOR);

        draw_rectangle(
            node.view_settings.staging_rect.x,
            node.view_settings.staging_rect.y,
            node.view_settings.staging_rect.w,
            node.view_settings.staging_rect.h,
            view::STAGING_BACKGROUND_COLOR,
        );

        draw_rectangle(
            node.view_settings.board_rect.x,
            node.view_settings.board_rect.y,
            node.view_settings.board_rect.w,
            node.view_settings.board_rect.h,
            view::BOARD_BACKGROUND_COLOR,
        );

        draw_rectangle(
            node.view_settings.preview_rect.x,
            node.view_settings.preview_rect.y,
            node.view_settings.preview_rect.w,
            node.view_settings.preview_rect.h,
            view::PREVIEW_BACKGROUND_COLOR,
        );

        draw_rectangle(
            node.view_settings.hold_rect.x,
            node.view_settings.hold_rect.y,
            node.view_settings.hold_rect.w,
            node.view_settings.hold_rect.h,
            view::HOLD_BACKGROUND_COLOR,
        );

        for (y, slots_x) in node.board.slots.iter().enumerate() {
            for (x, slot) in slots_x.iter().enumerate() {
                match slot {
                    SlotState::Locked(rtype) => {
                        // draw the block
                        let rect = board_block_rect([x as i32, y as i32], &node.view_settings);
                        draw_rectangle(rect.x, rect.y, rect.w, rect.h, rtype.color());
                    }
                    _ => {}
                }
            }
        }

        for rustomino in node.board.current_rustomino.iter() {
            for slot in rustomino.board_slots() {
                // display the preview
                // draw the block
                let rect = board_block_rect([slot[0], slot[1]], &node.view_settings);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    rustomino.rustomino_type.color(),
                );
            }
        }

        for ghost in node.board.ghost_rustomino.iter() {
            for block in ghost.board_slots() {
                // draw the block
                let rect = board_block_rect([block[0], block[1]], &node.view_settings);
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, view::GHOST_COLOR);
            }
        }
    }

    fn update(mut node: scene::RefMut<Self>)
    where
        Self: Sized,
    {
        let now = get_time();
        let delta_time = now - node.last_update;

        match node.game_state {
            GameState::Menu => todo!(),
            GameState::Playing => {
                // check board ready for the next rustomino
                if node.board.ready_for_next() {
                    // TODO: move this whole block to a fn
                    // take the next rustomino
                    // unwrap should be safe here
                    let current_rustomino = node.next_rustomino.take().unwrap();
                    // we used next_rustomino so we need to replace it
                    node.set_next_rustomino();
                    // add the next rustomino to the board
                    // game over if it can't be placed without a collision
                    if !node.board.add_new_rustomino(current_rustomino) {
                        node.game_over();
                    }
                }
                node.handle_inputs(delta_time);
                // Apply "gravity" to move the current rustomino down the board
                // or if it can't move lock it
                node.gravity_time_accum += delta_time;
                if node.gravity_time_accum >= node.gravity_delay {
                    node.gravity_time_accum = 0.0;
                    node.gravity_tick();
                }

                // increase the game level every LINES_PER_LEVEL
                if node.completed_lines > node.game_level * LINES_PER_LEVEL {
                    node.increase_game_level();
                }
            }
            GameState::GameOver => todo!(),
        }
        node.last_update = now;
    }
}

// pub fn draw(&mut self, controller: &RustrisGame) {
//     clear(BACKGROUND_COLOR, g);

//     match controller.game_state {
//         crate::controller::GameState::Menu => {}
//         crate::controller::GameState::Playing => {
//             self.draw_playing_background(ctx, g);
//             self.draw_playing_foreground(controller, ctx, g);
//             self.draw_overlay(controller, ctx, g);
//         }
//         crate::controller::GameState::GameOver => {}
//     }
//     // playing game state would be
//     // display the rustris board
//     // display the score
//     // display the level
// }

// fn draw_overlay(&mut self, controller: &RustrisGame) {
//     text(
//         [1.0, 1.0, 1.0, 1.0],
//         18,
//         "Rustris",
//         &mut self.glyph_cache,
//         ctx.transform.trans(
//             self.settings.title_label_pos[0],
//             self.settings.title_label_pos[1],
//         ),
//         g,
//     )
//     .expect("unable to render text");

//     text(
//         [1.0, 1.0, 1.0, 1.0],
//         18,
//         "Level:",
//         &mut self.glyph_cache,
//         ctx.transform.trans(
//             self.settings.level_label_pos[0],
//             self.settings.level_label_pos[1],
//         ),
//         g,
//     )
//     .expect("unable to render text");

//     text(
//         [1.0, 1.0, 1.0, 1.0],
//         18,
//         &controller.game_level.to_string(),
//         &mut self.glyph_cache,
//         ctx.transform
//             .trans(self.settings.level_pos[0], self.settings.level_pos[1]),
//         g,
//     )
//     .expect("unable to render text");

//     text(
//         [1.0, 1.0, 1.0, 1.0],
//         18,
//         "Score:",
//         &mut self.glyph_cache,
//         ctx.transform.trans(
//             self.settings.score_label_pos[0],
//             self.settings.score_label_pos[1],
//         ),
//         g,
//     )
//     .expect("unable to render text");

//     text(
//         [1.0, 1.0, 1.0, 1.0],
//         18,
//         &controller.score.to_string(),
//         &mut self.glyph_cache,
//         ctx.transform
//             .trans(self.settings.score_pos[0], self.settings.score_pos[1]),
//         g,
//     )
//     .expect("unable to render text");
// }

// fn draw_playing_foreground(&self, controller: &RustrisGame) {

//     // draw the board state
//     controller.board.draw(&self.settings, ctx, g);

//     // draw next rustomino
//     if let Some(rustomino) = controller.next_rustomino.as_ref() {
//         for block in rustomino.blocks {
//             // piece hold background
//             Rectangle::new(rustomino_color(rustomino.rustomino_type)).draw(
//                 next_block_rect(block, &self.settings),
//                 &ctx.draw_state,
//                 ctx.transform,
//                 g,
//             );
//         }
//     }

//     // draw held rustomino
//     if let Some(rustomino) = controller.hold_rustomino.as_ref() {
//         for block in rustomino.blocks {
//             // piece hold background
//             Rectangle::new(rustomino_color(rustomino.rustomino_type)).draw(
//                 hold_block_rect(block, &self.settings),
//                 &ctx.draw_state,
//                 ctx.transform,
//                 g,
//             );
//         }
//     }
// }

fn next_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect {
    // block[x,y] absolute units
    let x = settings.preview_rect.x
        + (block[0] as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING))
        + 1.0;
    // get bottom left of board_rect
    let y = settings.preview_rect.y + settings.preview_rect.h
        - (block[1] as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING));

    Rect::new(x, y, view::BLOCK_SIZE, view::BLOCK_SIZE)
}

fn hold_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect {
    // block[x,y] absolute units
    let x =
        settings.hold_rect.x + (block[0] as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING)) + 1.0;
    // get bottom left of board_rect
    let y = settings.hold_rect.y + settings.hold_rect.h
        - (block[1] as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING));

    Rect::new(x, y, view::BLOCK_SIZE, view::BLOCK_SIZE)
}

fn board_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect {
    // block[x,y] absolute units
    let x = settings.staging_rect.x
        + (block[0] as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING))
        + 1.0;
    // get bottom left of board_rect
    let y = settings.board_rect.y + settings.board_rect.h
        - ((block[1] + 1) as f32 * (view::BLOCK_SIZE + view::BLOCK_PADDING))
        - 1.0;

    Rect::new(x, y, view::BLOCK_SIZE, view::BLOCK_SIZE)
}
