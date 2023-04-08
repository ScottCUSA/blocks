use ggez::{
    audio,
    event::EventHandler,
    glam::IVec2,
    graphics::{self, Color},
    input::keyboard::KeyCode,
    input::keyboard::KeyInput,
    Context, GameResult,
};
use strum::IntoEnumIterator;

use crate::{
    controls::{self, Control, GameControls},
    playfield::{RustrisPlayfield, TranslationDirection, PLAYFIELD_SIZE},
    rustomino::{Rotation, Rustomino, RustominoBag, RustominoState},
    view::{self, BACKGROUND_COLOR},
};

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

// ASSET CONSTANTS
const ASSETS_FOLDER: &str = "assets";
const MUSIC_VOL: f32 = 0.05;

pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

pub struct Assets {
    pub music_1: audio::Source,
    pub game_over: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        // load background music
        let music_1 = audio::Source::new(ctx, "/music_1.ogg")?;

        // load game sound effects
        let game_over = audio::Source::new(ctx, "/game_over.ogg")?;

        Ok(Assets { music_1, game_over })
    }
}

pub struct RustrisGame {
    pub playfield: RustrisPlayfield,
    pub next_rustomino: Option<Rustomino>,
    pub held_rustomino: Option<Rustomino>,
    pub state: GameState,
    pub level: usize,
    pub score: usize,
    pub assets: Assets,
    pub controls: Option<GameControls>,
    rustomino_bag: RustominoBag,
    gravity_delay: f64, // time between gravity ticks
    total_lines_cleared: usize,
    hold_used: bool, // if user has held a rustomino, resets on lock
    lockdown_resets: u32,
    music_volume: f32,
}

impl RustrisGame {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // load font
        ctx.gfx
            .add_font("04b30", graphics::FontData::from_path(ctx, "/04b30.ttf")?);

        let assets = Assets::new(ctx)?;
        let control_state = Some(GameControls::default());
        let playfield = RustrisPlayfield::new();

        let s = RustrisGame {
            playfield,
            next_rustomino: None,
            held_rustomino: None,
            state: GameState::Menu, // Start the game at the menu screen
            level: STARTING_LEVEL,
            assets,
            controls: control_state,
            score: 0,
            rustomino_bag: RustominoBag::new(),
            gravity_delay: gravity_delay(0),
            total_lines_cleared: 0,
            hold_used: false,
            lockdown_resets: 0,
            music_volume: 0.,
        };

        Ok(s)
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
                    // the block can't move down so it's state becomes "Lockdown"
                    // If this block has been in Lockdown state before
                    if self.lockdown_resets > 0 {
                        // hitting the deck again causes a lockdown reset
                        self.lockdown_resets += 1;
                        log::debug!("incrementing lockdown resets: {}", self.lockdown_resets);
                    }
                    log::debug!("setting active rustomino state to lockdown");

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
                if self.lockdown_resets >= LOCKDOWN_MAX_RESETS
                    && !self.playfield.active_can_fall() =>
            {
                // if the user has exceeded the maximum number of resets
                // lock the block
                log::info!("maximum lockdown resets exceeded");
                self.lock();
            }
            RustominoState::Lockdown { time }
                if time + delta_time >= LOCKDOWN_MAX_TIME && !self.playfield.active_can_fall() =>
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

    fn ensure_next_rustomino(&mut self) {
        // make sure next_rustomino is available
        if self.next_rustomino.is_none() {
            self.next_rustomino = Some(self.rustomino_bag.get_next_rustomino());
        }
    }

    fn ready_playfield(&mut self) {
        // make sure next_rustomino is available
        self.ensure_next_rustomino();
        // check to see if the playfield is ready for the next rustomino
        if self.playfield.ready_for_next() {
            log::debug!("playfield is ready for next rustomino");
            // take the next rustomino
            let active_rustomino = self.next_rustomino.take().unwrap();
            // this makes sure next_rustomino is set
            self.ensure_next_rustomino();
            // add the next rustomino to the playfield
            if !self.playfield.set_active(active_rustomino) {
                // game over if it can't be placed without a collision
                self.game_over();
            }
        }
    }

    fn translate(&mut self, direction: TranslationDirection) {
        log::info!("translate called, direction: {:?}", direction);
        if self.playfield.translate_active(direction) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    fn rotate(&mut self, rotation: Rotation) {
        log::info!("rotate called, direction: {:?}", rotation);
        if self.playfield.rotate_active(rotation) {
            self.increment_lockdown_resets();
        }
        log::trace!("playfield:\n{}", self.playfield);
    }

    // performs a soft drop
    fn soft_drop(&mut self) {
        log::info!("soft drop called");
        if !self.playfield.translate_active(TranslationDirection::Down) {
            log::info!("soft drop called when block is on stack");
            self.lock();
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

        // take active_rustomino and make it the hold_rustomino
        self.held_rustomino = self.playfield.take_active();

        // trigger game over in the unusual circumstance
        // a collision with a locked block occurs
        // when the hold piece is added to the board
        if !self.playfield.set_active(rustomino.reset()) {
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

    fn new_game(&mut self) {
        self.playfield = RustrisPlayfield::new();
        self.next_rustomino = None;
        self.held_rustomino = None;
        self.state = GameState::Menu; // Start the game at the menu screen
        self.level = STARTING_LEVEL;
        self.controls.as_mut().unwrap().clear_inputs();
        self.score = 0;
        self.rustomino_bag = RustominoBag::new();
        self.gravity_delay = gravity_delay(0);
        self.total_lines_cleared = 0;
        self.hold_used = false;
        self.lockdown_resets = 0;
        self.music_volume = 0.;
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
    fn translate_left(&mut self) {
        self.translate(TranslationDirection::Left);
    }
    fn translate_right(&mut self) {
        self.translate(TranslationDirection::Right);
    }
    fn rotate_cw(&mut self) {
        self.rotate(Rotation::Cw);
    }
    fn rotate_ccw(&mut self) {
        self.rotate(Rotation::Ccw);
    }
    // returns a closure which handles the provided
    // control for the game
    pub fn control_handler(&mut self, control: &Control) -> fn(&mut RustrisGame) {
        match control {
            Control::Left => RustrisGame::translate_left,
            Control::Right => RustrisGame::translate_right,
            Control::RotateCW => RustrisGame::rotate_cw,
            Control::RotateCCW => RustrisGame::rotate_ccw,
            Control::SoftDrop => RustrisGame::soft_drop,
            Control::HardDrop => RustrisGame::hard_drop,
            Control::Hold => RustrisGame::hold,
        }
    }
    pub fn handle_key_down(&mut self, input: &KeyInput) {
        let Some(mut control_state) = self.controls.take() else {
            return
        };
        // iterate through the controls
        for (control, control_inputs) in &control_state.inputs_map {
            if input.keycode == control_inputs.0 || input.keycode == control_inputs.1 {
                // control_state
                //     .states_map
                //     .entry(*control)
                //     .and_modify(|e| *e = InputState::Down(0.0));
                if let Some(input_state) = control_state.states_map.get_mut(control) {
                    if *input_state == controls::InputState::Up {
                        *input_state = controls::InputState::Down(0.0);
                        // call the game function for this input
                        self.control_handler(control)(self);
                    }
                }
            }
        }
        self.controls = Some(control_state);
    }

    pub fn handle_key_up(&mut self, input: &KeyInput) {
        let Some(mut control_state) = self.controls.take() else {
            return
        };
        // iterate through the controls
        for (control, control_inputs) in &control_state.inputs_map {
            // if the input matches one of a controls inputs set the input state to Up
            if input.keycode == control_inputs.0 || input.keycode == control_inputs.1 {
                control_state
                    .states_map
                    .entry(*control)
                    .and_modify(|e| *e = controls::InputState::Up);
            }
        }
        self.controls = Some(control_state);
    }

    // Some of the games controls allow repeating their actions
    // when the user holds their inputs
    // This handles updating the state of these inputs
    // as well as calling game functions when appropriate
    pub fn handle_held_inputs(&mut self, delta_time: f64) {
        let Some(mut control_state) = self.controls.take() else {
            return
        };
        // iterate through the controls
        for control in Control::iter() {
            control_state
                .states_map
                .entry(control) // modify in place
                .and_modify(|e| match e {
                    controls::InputState::Down(down_time) => {
                        // check to see if the key is repeatable
                        // and if the down time is longer than the action delay for this input
                        if let Some(action_delay) = control.action_delay() {
                            *down_time += delta_time;
                            if *down_time >= action_delay {
                                *e = controls::InputState::Held(0.);
                                self.control_handler(&control)(self);
                            }
                        }
                    }
                    // if the input state is held, add delta time to the held time
                    controls::InputState::Held(held_time) => {
                        *held_time += delta_time;
                    }
                    _ => (),
                });
            if let Some(state) = control_state.states_map.get_mut(&control) {
                // if this input is in a held state
                if let controls::InputState::Held(held_time) = state {
                    // check if held was just set
                    if *held_time == 0. {
                        // call the game control handler function
                        self.control_handler(&control)(self);
                    }
                    // check to see if the key is repeatable
                    // and if the key has been held longer than the repeat delay for the input
                    if let Some(action_repeat_delay) = control.action_repeat_delay() {
                        if *held_time >= action_repeat_delay {
                            // reset the held state time
                            *state = controls::InputState::Held(0.);
                            // call the game control handler function
                            self.control_handler(&control)(self);
                        }
                    }
                }
            }
        }
        self.controls = Some(control_state);
    }
}

impl EventHandler for RustrisGame {
    fn update(&mut self, ctx: &mut ggez::Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        // limit game to 60fps
        while ctx.time.check_update_time(DESIRED_FPS) {
            let delta_time = ctx.time.delta().as_secs_f64();

            // handle the game states
            match self.state {
                GameState::Menu => {}
                GameState::Playing => {
                    // pause the game immediately
                    // clear all other inputs and continue
                    // if is_key_pressed(KeyCode::Escape) {
                    //     game.pause();
                    //     controls.clear_inputs();
                    // } else {
                    self.ready_playfield();
                    self.handle_held_inputs(delta_time);
                    self.playing_update(delta_time);
                    // }
                }
                GameState::Paused => {}
                GameState::GameOver => {}
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, BACKGROUND_COLOR);

        // Draw code here...
        view::draw(self, ctx, &mut canvas)?;

        // println!("{:?}", view::VIEW_SETTINGS);

        canvas.finish(ctx)?;

        ggez::timer::yield_now();
        Ok(())
    }

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        repeated: bool,
    ) -> GameResult {
        match self.state {
            GameState::Menu => {
                // handle the user's inputs
                if input.keycode == Some(KeyCode::Return) && !repeated {
                    self.controls.as_mut().unwrap().clear_inputs();
                    self.resume();
                }
            }
            GameState::Playing => {
                // pause the game immediately
                // clear all other inputs and continue
                if input.keycode == Some(KeyCode::Escape) {
                    self.pause();
                    self.controls.as_mut().unwrap().clear_inputs();
                } else {
                    self.handle_key_down(&input);
                }
            }
            GameState::Paused => {
                if input.keycode == Some(KeyCode::Escape) && !repeated {
                    self.controls.as_mut().unwrap().clear_inputs();
                    self.resume();
                }
            }
            GameState::GameOver => {
                if input.keycode == Some(KeyCode::Return) && !repeated {
                    self.controls.as_mut().unwrap().clear_inputs();
                    self.new_game();
                }
            }
        }
        controls::handle_global_inputs(&input, &mut self.music_volume);
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        match self.state {
            GameState::Menu => {}
            GameState::Playing => {
                // pause the game immediately
                // clear all other inputs and continue
                if input.keycode == Some(KeyCode::Escape) {
                    self.pause();
                    self.controls.as_mut().unwrap().clear_inputs();
                } else {
                    self.handle_key_up(&input);
                }
            }
            GameState::Paused => {}
            GameState::GameOver => {}
        }
        Ok(())
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
            _ => panic!("impossible number of lines cleared"),
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

// // run the game
// pub async fn run() {
//     log::info!("startup: initializing Rustris;");

//     // initialize the game and control states
//     let mut game = RustrisGame::new(RustrisPlayfield::new());
//     let mut controls = controls::ControlStates::default();

//     log::info!("loading Resources");
//     // find our assets path
//     let assets_path = find_folder::Search::ParentsThenKids(2, 2)
//         .for_folder(ASSETS_FOLDER)
//         .expect("unable to find assets folder");

//     // load the font
//     let font_path = assets_path.join("04b30.ttf");
//     log::info!("loading font: {:?}", font_path);
//     let font = load_ttf_font(&font_path.to_string_lossy())
//         .await
//         .expect("unable to load font");

//     // configure UI fonts
//     let font_20pt = TextParams {
//         font,
//         font_size: 20,
//         ..Default::default()
//     };
//     let font_30pt = TextParams {
//         font,
//         font_size: 30,
//         ..Default::default()
//     };

//     // load the background music
//     let background_path = assets_path.join("background.ogg");
//     log::info!("loading background music: {:?}", background_path);
//     let background_music = load_sound(&background_path.to_string_lossy())
//         .await
//         .expect("unable to load background music");

//     // play background music
//     let mut music_volume = MUSIC_VOL;
//     log::info!("playing background music at volume: {music_volume}");
//     play_sound(
//         background_music,
//         PlaySoundParams {
//             looped: true,
//             volume: music_volume,
//         },
//     );

//     let mut last_update = get_time();

//     loop {
//         clear_background(view::BACKGROUND_COLOR);

//         // handle global controls
//         handle_global_inputs(&background_music, &mut music_volume);

//         let now = get_time();
//         let delta_time = now - last_update;

//         // handle the game states
//         match game.state {
//             GameState::Menu => {
//                 // handle the user's inputs
//                 if is_key_pressed(KeyCode::Enter) {
//                     controls.clear_inputs();
//                     game.resume();
//                 }
//             }
//             GameState::Playing => {
//                 // pause the game immediately
//                 // clear all other inputs and continue
//                 if is_key_pressed(KeyCode::Escape) {
//                     game.pause();
//                     controls.clear_inputs();
//                 } else {
//                     game.ready_playfield();
//                     handle_playing_inputs(&mut controls, &mut game);
//                     handle_held_playing_inputs(&mut controls, &mut game, delta_time);
//                     game.playing_update(delta_time);
//                 }
//             }
//             GameState::Paused => {
//                 if is_key_pressed(KeyCode::Escape) {
//                     controls.clear_inputs();
//                     game.resume();
//                 }
//             }
//             GameState::GameOver => {
//                 if is_key_pressed(KeyCode::Enter) {
//                     controls.clear_inputs();
//                     game = game.new_game();
//                 }
//             }
//         }

//         // draw the menus, game, overlays, etc.
//         view::draw(&game, &font_20pt, &font_30pt);

//         last_update = get_time();

//         next_frame().await;
//     }
// }
