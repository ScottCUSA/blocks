use ggez::{
    audio::{self, SoundSource},
    event::EventHandler,
    glam::IVec2,
    graphics::{self},
    input::keyboard::KeyCode,
    input::keyboard::KeyInput,
    Context, GameResult,
};

use crate::{
    controls::{self, Control, GameControls},
    draw::{self, BACKGROUND_COLOR},
    menus::{self, Menu},
    playfield::{RustrisPlayfield, TranslationDirection, PLAYFIELD_SIZE},
    rustomino::{Rotation, Rustomino, RustominoBag, RustominoState},
    util::variants_equal,
};

use std::f64::consts::E;

// GAMEPLAY CONSTANTS
const GRAVITY_NUMERATOR: f64 = 1.0;
const GRAVITY_FACTOR: f64 = 0.1; // used to slow or increase gravity factor
const STARTING_LEVEL: usize = 20;
const LINES_PER_LEVEL: usize = 10; // number of lines that need to be cleared before level advances
const LOCKDOWN_DELAY: f64 = 0.5; // how long to wait before locking block (Tetris Guideline)
const LOCKDOWN_MAX_RESETS: u32 = 15; // maximum number of times the lockdown timer can be reset (Tetris Guideline)

// SCORING CONSTANTS
const SINGLE_LINE_SCORE: usize = 100;
const TRIPLE_LINE_SCORE: usize = 500;
const DOUBLE_LINE_SCORE: usize = 300;
const RUSTRIS_SCORE: usize = 800;

// ASSET CONSTANTS
const MUSIC_VOL: f32 = 0.0;
const MUSIC_VOLUME_CHANGE: f32 = 0.005;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Options,
    Quit,
}

pub struct Assets {
    pub music_1: audio::Source,
    pub game_over: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        // load background music
        let mut music_1 = audio::Source::new(ctx, "/music_1.ogg")?;
        music_1.set_volume(MUSIC_VOL);
        music_1.set_repeat(true);

        // load game sound effects
        let game_over = audio::Source::new(ctx, "/game_over.ogg")?;

        Ok(Assets { music_1, game_over })
    }
}

pub struct RustrisState {
    pub playfield: RustrisPlayfield,
    pub next_rustomino: Option<Rustomino>,
    pub held_rustomino: Option<Rustomino>,
    pub previous_state: GameState,
    pub state: GameState,
    pub level: usize,
    pub score: usize,
    pub assets: Assets,
    pub controls: Option<GameControls>,
    menu_state: menus::MenuState,
    paused_state: menus::PausedState,
    view_settings: draw::ViewSettings,
    rustomino_bag: RustominoBag,
    gravity_delay: f64, // time between gravity ticks
    total_lines_cleared: usize,
    hold_used: bool, // if user has held a rustomino, resets on lock
    lockdown_resets: u32,
    music_volume: f32,
}

impl RustrisState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        log::info!("Loading game resources");
        // load font
        ctx.gfx
            .add_font("04b30", graphics::FontData::from_path(ctx, "/04b30.ttf")?);

        // load game resources
        let mut assets = Assets::new(ctx)?;
        assets.music_1.play_detached(ctx)?;

        let control_state = Some(GameControls::default());
        let playfield = RustrisPlayfield::new();

        // get the window size
        let (width, height) = ctx.gfx.drawable_size();

        let s = RustrisState {
            playfield,
            next_rustomino: None,
            held_rustomino: None,
            previous_state: GameState::Menu,
            state: GameState::Menu, // Start the game at the menu screen
            level: STARTING_LEVEL,
            assets,
            controls: control_state,
            menu_state: menus::MenuState::new(),
            paused_state: menus::PausedState::new(),
            view_settings: draw::ViewSettings::new(width, height),
            score: 0,
            rustomino_bag: RustominoBag::new(),
            gravity_delay: gravity_delay(STARTING_LEVEL),
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
                    // if the block can't fall, set it's state to lockdown
                    self.set_lockdown();
                }
            }
            RustominoState::Falling { time } => {
                self.playfield.set_active_state(RustominoState::Falling {
                    time: time + delta_time,
                });
            }
            RustominoState::Lockdown { time }
                if self.lockdown_resets >= LOCKDOWN_MAX_RESETS
                    && !self.playfield.active_can_fall() =>
            {
                // accumulate lockdown time
                self.playfield.set_active_state(RustominoState::Lockdown {
                    time: time + delta_time,
                });

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
        log::info!("setting active rustomino state to lockdown");
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

    fn ready_playfield(&mut self) -> bool {
        // check to see if the playfield is ready for the next rustomino
        if !self.playfield.ready_for_next() {
            return true;
        }

        log::debug!("playfield is ready for next rustomino");

        // get the next rustomino
        let active_rustomino = self.get_next_rustomino();

        // add the next rustomino to the playfield
        if !self.playfield.set_active(active_rustomino) {
            log::info!("couldn't add next piece to board, collided with locked block");
            // game over if it can't be placed without a collision
            self.game_over();
            return false;
        }

        true
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

    fn new_game(&mut self) {
        self.playfield = RustrisPlayfield::new();
        self.next_rustomino = None;
        self.held_rustomino = None;
        self.state = GameState::Menu; // Start the game at the menu screen
        self.level = STARTING_LEVEL;
        self.score = 0;
        self.rustomino_bag = RustominoBag::new();
        self.gravity_delay = gravity_delay(STARTING_LEVEL);
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
            log::warn!("no active rustomino");
            return;
        };

        log::info!("locking block type: {:?}", rustomino.rtype);
        log::debug!("blocks: {:?}", rustomino.playfield_slots());

        // if the block we've been asked to lock is fully
        // out of bounds the game is over
        if fully_out_of_bounds(&rustomino.playfield_slots()) {
            log::info!("block we are locking is fully out of playfield");
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
    pub fn control_handler(&mut self, control: &Control) -> fn(&mut RustrisState) {
        match control {
            Control::Left => RustrisState::translate_left,
            Control::Right => RustrisState::translate_right,
            Control::RotateCW => RustrisState::rotate_cw,
            Control::RotateCCW => RustrisState::rotate_ccw,
            Control::SoftDrop => RustrisState::soft_drop,
            Control::HardDrop => RustrisState::hard_drop,
            Control::Hold => RustrisState::hold,
        }
    }

    fn menu_item_selected(&mut self) {
        if self.menu_state.selected() == 0 {
            self.resume();
        } else if self.menu_state.selected() == 1 {
            // self.state = GameState::Options;
        } else if self.menu_state.selected() == 2 {
            self.state = GameState::Quit;
        }
        self.menu_state.reset_selection();
    }

    fn paused_item_selected(&mut self) {
        if self.paused_state.selected() == 0 {
            self.resume();
        } else if self.paused_state.selected() == 1 {
            // self.state = GameState::Options;
        } else if self.paused_state.selected() == 2 {
            self.new_game();
        } else if self.paused_state.selected() == 3 {
            self.state = GameState::Quit;
        }
        self.paused_state.reset_selection();
    }
}

impl EventHandler for RustrisState {
    fn update(&mut self, ctx: &mut ggez::Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        // limit game to 60fps
        while ctx.time.check_update_time(DESIRED_FPS) {
            let delta_time = 1.0 / (DESIRED_FPS as f64);
            // handle the game states
            match self.state {
                GameState::Menu => {
                    self.previous_state = GameState::Menu;
                }
                GameState::Playing => {
                    if self.ready_playfield() {
                        self.playing_update(delta_time);
                    }
                    self.previous_state = GameState::Playing;
                }
                GameState::Paused => {
                    self.previous_state = GameState::Paused;
                }
                GameState::GameOver if self.previous_state != self.state => {
                    // play game over sound
                    self.assets.game_over.play(ctx)?;
                    self.previous_state = GameState::GameOver;
                }
                GameState::GameOver => {}
                GameState::Options => {}
                GameState::Quit => ctx.request_quit(),
            }
            self.previous_state = self.state;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, BACKGROUND_COLOR);
        // handle the game states

        match self.state {
            GameState::Menu => {
                draw::draw_menu(ctx, &mut canvas, &self.menu_state, &self.view_settings)?;
            }
            GameState::Playing => {
                draw::draw_playing(
                    ctx,
                    &mut canvas,
                    &self.playfield,
                    &self.next_rustomino,
                    &self.held_rustomino,
                    &self.view_settings,
                    false,
                )?;
                draw::draw_playing_overlay(
                    ctx,
                    &mut canvas,
                    self.level,
                    self.score,
                    &self.view_settings,
                )?;
            }
            GameState::Paused => {
                draw::draw_playing(
                    ctx,
                    &mut canvas,
                    &self.playfield,
                    &self.next_rustomino,
                    &self.held_rustomino,
                    &self.view_settings,
                    false,
                )?;
                draw::draw_paused(ctx, &mut canvas, &self.paused_state, &self.view_settings)?;
                // draw::draw_playing_overlay(game.level, game.score);
                // draw::draw_paused();
                // draw::draw_help_text();
            }
            GameState::GameOver => {
                draw::draw_playing_backgound(ctx, &mut canvas, &self.view_settings)?;
                draw::draw_playing(
                    ctx,
                    &mut canvas,
                    &self.playfield,
                    &self.next_rustomino,
                    &self.held_rustomino,
                    &self.view_settings,
                    true,
                )?;
                draw::draw_playing_overlay(
                    ctx,
                    &mut canvas,
                    self.level,
                    self.score,
                    &self.view_settings,
                )?;
                draw::draw_gameover(ctx, &mut canvas, &self.view_settings.view_rect)?;
            }
            GameState::Options => todo!(),
            GameState::Quit => {}
        }

        canvas.finish(ctx)?;

        ggez::timer::yield_now();
        Ok(())
    }

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> GameResult {
        match self.state {
            GameState::Menu => {
                // handle the user's inputs
                if input.keycode == Some(KeyCode::Return) && !repeated {
                    self.menu_item_selected();
                }
                if input.keycode == Some(KeyCode::Escape) && !repeated {
                    self.state = GameState::Quit;
                }
                if input.keycode == Some(KeyCode::Up) && !repeated {
                    self.menu_state.previous();
                }
                if input.keycode == Some(KeyCode::Down) && !repeated {
                    self.menu_state.next();
                }
            }
            GameState::Playing => {
                // pause the game immediately
                // clear all other inputs and continue
                if input.keycode == Some(KeyCode::Escape) {
                    self.pause();
                }
            }
            GameState::Paused => {
                if input.keycode == Some(KeyCode::Escape) && !repeated {
                    self.paused_state.reset_selection();
                    self.resume();
                }
                if input.keycode == Some(KeyCode::Return)
                    || input.keycode == Some(KeyCode::NumpadEnter) && !repeated
                {
                    self.paused_item_selected();
                }
                if input.keycode == Some(KeyCode::Up) && !repeated {
                    self.paused_state.previous();
                }
                if input.keycode == Some(KeyCode::Down) && !repeated {
                    self.paused_state.next();
                }
            }
            GameState::GameOver => {
                self.new_game();
            }
            GameState::Options => todo!(),
            GameState::Quit => {}
        }
        handle_global_inputs(&input, &mut self.music_volume);
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        match self.state {
            GameState::Menu => {}
            GameState::Playing => {}
            GameState::Paused => {}
            GameState::GameOver => {}
            GameState::Options => {}
            GameState::Quit => {}
        }
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.view_settings = draw::ViewSettings::new(width, height);
        Ok(())
    }

    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) -> Result<(), ggez::GameError> {
        if !gained && self.state == GameState::Playing {
            self.pause();
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

// returns a closure which handles the provided
// control for the game
fn control_handler<'a>(control: &'a Control, game: &'a mut RustrisState) -> Box<dyn FnMut() + 'a> {
    match *control {
        Control::Left => Box::new(|| game.translate(TranslationDirection::Left)),
        Control::Right => Box::new(|| game.translate(TranslationDirection::Right)),
        Control::RotateCW => Box::new(|| game.rotate(Rotation::Cw)),
        Control::RotateCCW => Box::new(|| game.rotate(Rotation::Ccw)),
        Control::SoftDrop => Box::new(|| game.soft_drop()),
        Control::HardDrop => Box::new(|| game.hard_drop()),
        Control::Hold => Box::new(|| game.hold()),
    }
}

pub fn handle_global_inputs(input: &KeyInput, music_volume: &mut f32) {
    // volume down
    if input.keycode == Some(KeyCode::Minus) || input.keycode == Some(KeyCode::NumpadSubtract) {
        *music_volume -= MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        log::debug!("volume decrease {}", music_volume);
    }
    // volume up
    if input.keycode == Some(KeyCode::Equals) || input.keycode == Some(KeyCode::NumpadAdd) {
        *music_volume += MUSIC_VOLUME_CHANGE;
        *music_volume = music_volume.clamp(0.0, 1.0);
        log::debug!("volume increase {}", music_volume);
    }
}
