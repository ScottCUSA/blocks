//! the playfield, pieces, scoring and the rules that move them

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use strum::IntoEnumIterator;

use super::{Screen, LINES_PER_LEVEL, STARTING_LEVEL};
use crate::audio::{PlaySfx, Sfx};
use crate::controls::{Control, GameControls, InputState};
use crate::gfx::{Color, IVec2};
use crate::level_progress::LevelProgress;
use crate::playfield::{Playfield, TranslationDirection, PLAYFIELD_SIZE};
use crate::rustomino::{Rotation, Rustomino, RustominoBag, RustominoState};
use crate::score_display::ScoreDisplay;

const FAST_GRAVITY_DELAY: f64 = 0.05; // gravity trail on every row at or below this
const MIN_GRAVITY_DELAY: f64 = 1.0 / 1200.0; // 20 rows per 60th of a second
const STANDARD_TOP_LEVEL: usize = 15; // standard mode stops speeding up here
const LOCKDOWN_DELAY: f64 = 0.5; // how long to wait before locking block (guideline)
pub const LINE_CLEAR_DELAY: f64 = 0.4; // how long completed lines stay before being removed
const LOCKDOWN_MAX_RESETS: u32 = 15; // maximum number of times the lockdown timer can be reset (guideline)

// SCORING CONSTANTS
const SINGLE_LINE_SCORE: usize = 100;
const TRIPLE_LINE_SCORE: usize = 500;
const DOUBLE_LINE_SCORE: usize = 300;
const QUAD_SCORE: usize = 800;
const SOFT_DROP_SCORE: usize = 1; // per row
const HARD_DROP_SCORE: usize = 2; // per row

/// gravity curve, fixed for a game
#[derive(Resource, Debug, PartialEq, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Standard, // speeds up until level 15
    Hard, // speeds up to instant drops at level 19
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Standard, Mode::Hard];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Standard => "Standard",
            Mode::Hard => "Hard",
        }
    }

    /// ghost piece and hold
    pub fn assists(self) -> bool {
        self != Mode::Hard
    }
}

/// what happened on the board, for effects, background and sound
#[derive(Message, Debug, Clone, Copy)]
pub enum BoardEvent {
    /// locked where gravity left it
    Locked {
        blocks: [IVec2; 4],
        color: Color,
    },
    HardDropped {
        from: [IVec2; 4],
        to: [IVec2; 4],
        color: Color,
    },
    /// fell more than a row in one tick
    GravityDropped {
        from: [IVec2; 4],
        to: [IVec2; 4],
        color: Color,
    },
    SoftDropped {
        blocks: [IVec2; 4],
        color: Color,
    },
    /// lines started clearing, the playfield holds which
    LinesCleared {
        lines: usize,
    },
    LevelUp,
    ToppedOut {
        level: usize,
    },
}

/// the playfield and the pieces around it
#[derive(Resource)]
pub struct Board {
    pub playfield: Playfield,
    pub next: Option<Rustomino>,
    pub held: Option<Rustomino>,
    pub bag: RustominoBag,
    pub gravity_delay: f64, // time between gravity ticks
    pub hold_used: bool,    // if user has held a rustomino, resets on lock
    pub lockdown_resets: u32,
    pub lowest_row: i32,  // lowest row reached by the active rustomino
    pub topped_out: bool, // no more moves this game
}

impl Board {
    pub fn new(mode: Mode) -> Self {
        Board {
            playfield: Playfield::new(),
            next: None,
            held: None,
            bag: RustominoBag::new(),
            gravity_delay: gravity_delay(STARTING_LEVEL, mode),
            hold_used: false,
            lockdown_resets: 0,
            lowest_row: i32::MAX,
            topped_out: false,
        }
    }
}

/// score, level and their animated displays
#[derive(Resource)]
pub struct Score {
    pub level: usize,
    pub score: usize,
    pub lines: usize, // cleared this game
    pub score_display: ScoreDisplay,
    pub level_display: ScoreDisplay,
    pub level_progress: LevelProgress,
}

impl Default for Score {
    fn default() -> Self {
        Score {
            level: STARTING_LEVEL,
            score: 0,
            lines: 0,
            score_display: ScoreDisplay::new(0),
            level_display: ScoreDisplay::new(STARTING_LEVEL),
            level_progress: LevelProgress::new(LINES_PER_LEVEL, 0),
        }
    }
}

impl Score {
    /// score, level and progress bar animations
    pub fn update_displays(&mut self, delta_time: f64) {
        self.score_display.update(delta_time);
        self.level_display.update(delta_time);
        // the shown level ticks up with the bar's burst
        if self.level_progress.update(delta_time) {
            self.level_display.add(1);
        }
    }
}

/// the rules, acting on the board and score
#[derive(SystemParam)]
pub struct Play<'w> {
    pub board: ResMut<'w, Board>,
    pub score: ResMut<'w, Score>,
    pub mode: Res<'w, Mode>,
    pub controls: ResMut<'w, GameControls>,
    sfx: MessageWriter<'w, PlaySfx>,
    events: MessageWriter<'w, BoardEvent>,
    next_screen: ResMut<'w, NextState<Screen>>,
}

impl Play<'_> {
    fn play(&mut self, effect: Sfx) {
        self.sfx.write(PlaySfx(effect));
    }

    /// gravity and lockdown for the active piece
    pub fn playing_update(&mut self, delta_time: f64) {
        let Some(current_state) = self.board.playfield.get_active_state() else {
            return;
        };
        let can_fall = self.board.playfield.active_can_fall();
        let gravity_delay = self.board.gravity_delay;
        match current_state {
            // start lockdown as soon as the block lands
            RustominoState::Falling { .. } if !can_fall => {
                self.set_lockdown();
            }
            RustominoState::Falling { time } if time + delta_time >= gravity_delay => {
                // drop one row per elapsed gravity delay, carrying over the remainder
                let mut time = time + delta_time;
                let from = self
                    .board
                    .playfield
                    .active_rustomino
                    .as_ref()
                    .map(|r| r.playfield_slots());
                let mut rows = 0;
                // max gravity lands instantly, no midair frames
                let instant = gravity_delay <= MIN_GRAVITY_DELAY;
                while (instant || time >= gravity_delay) && self.board.playfield.active_can_fall() {
                    self.board.playfield.apply_gravity();
                    time = (time - gravity_delay).max(0.0);
                    rows += 1;
                }
                // streak fast falls and skipped rows
                if rows > 1 || (rows > 0 && gravity_delay <= FAST_GRAVITY_DELAY) {
                    if let (Some(from), Some(rustomino)) =
                        (from, &self.board.playfield.active_rustomino)
                    {
                        let (to, color) = (rustomino.playfield_slots(), rustomino.rtype.color());
                        self.events
                            .write(BoardEvent::GravityDropped { from, to, color });
                    }
                }
                self.update_lowest_row();
                self.board
                    .playfield
                    .set_active_state(RustominoState::Falling { time });
            }
            RustominoState::Falling { time } => {
                self.board
                    .playfield
                    .set_active_state(RustominoState::Falling {
                        time: time + delta_time,
                    });
            }
            RustominoState::Lockdown { .. }
                if self.board.lockdown_resets >= LOCKDOWN_MAX_RESETS && !can_fall =>
            {
                log::info!("maximum lockdown resets exceeded");
                self.lock(Sfx::Lock);
            }
            RustominoState::Lockdown { time }
                if time + delta_time >= LOCKDOWN_DELAY && !can_fall =>
            {
                log::info!("lockdown time expired");
                self.lock(Sfx::Lock);
            }
            RustominoState::Lockdown { time } => {
                self.board
                    .playfield
                    .set_active_state(RustominoState::Lockdown {
                        time: time + delta_time,
                    });
            }
        }
    }

    fn set_lockdown(&mut self) {
        // hitting the deck again causes a lockdown reset
        if self.board.lockdown_resets > 0 {
            self.board.lockdown_resets += 1;
            log::debug!(
                "incrementing lockdown resets: {}",
                self.board.lockdown_resets
            );
        }
        log::info!("setting active rustomino state to lockdown");
        self.board
            .playfield
            .set_active_state(RustominoState::Lockdown { time: 0. });
    }

    /// the next piece, refilling the preview from the bag
    pub(super) fn get_next_rustomino(&mut self) -> Rustomino {
        let board = &mut *self.board;
        let next = match board.next.take() {
            Some(rustomino) => rustomino,
            None => board.bag.get_next(),
        };
        board.next = Some(board.bag.get_next());
        next
    }

    /// spawn the next piece when the playfield is ready for it
    /// false on game over
    pub fn ready_playfield(&mut self) -> bool {
        if !self.board.playfield.ready_for_next() {
            return true;
        }
        log::debug!("playfield is ready for next rustomino");
        let active = self.get_next_rustomino();
        if !self.spawn(active) {
            log::info!("couldn't add next piece to board, collided with locked block");
            self.game_over();
            return false;
        }
        true
    }

    /// set the active rustomino, false on collision
    pub fn spawn(&mut self, rustomino: Rustomino) -> bool {
        self.board.lowest_row = i32::MAX;
        let ok = self.board.playfield.set_active(rustomino);
        self.update_lowest_row();
        ok
    }

    /// reset lockdown resets if the active rustomino reached a new lowest row
    fn update_lowest_row(&mut self) {
        let Some(row) = self.board.playfield.active_lowest_row() else {
            return;
        };
        if row < self.board.lowest_row {
            self.board.lowest_row = row;
            self.board.lockdown_resets = 0;
        }
    }

    pub fn translate(&mut self, direction: TranslationDirection) -> bool {
        let moved = self.board.playfield.translate_active(direction);
        if moved {
            self.play(Sfx::Move);
            self.increment_lockdown_resets();
        }
        moved
    }

    pub fn rotate(&mut self, rotation: Rotation) -> bool {
        let turned = self.board.playfield.rotate_active(rotation);
        if turned {
            self.play(Sfx::Rotate);
            // wall kicks can move the rustomino down
            self.update_lowest_row();
            self.increment_lockdown_resets();
        }
        turned
    }

    pub fn soft_drop(&mut self) {
        // landing is handled in playing_update
        if self
            .board
            .playfield
            .translate_active(TranslationDirection::Down)
        {
            if let Some(rustomino) = &self.board.playfield.active_rustomino {
                let (blocks, color) = (rustomino.playfield_slots(), rustomino.rtype.color());
                self.events.write(BoardEvent::SoftDropped { blocks, color });
            }
            self.update_lowest_row();
            self.score.score += SOFT_DROP_SCORE;
            self.score.score_display.add_quiet(SOFT_DROP_SCORE);
        }
    }

    pub fn hard_drop(&mut self) {
        // no active rustomino between lock and spawn
        if self.board.playfield.ready_for_next() {
            return;
        }
        if let Some(from) = self.board.playfield.hard_drop_active() {
            if let Some(rustomino) = &self.board.playfield.active_rustomino {
                let (to, color) = (rustomino.playfield_slots(), rustomino.rtype.color());
                let rows = (from[0].y - to[0].y) as usize;
                if rows > 0 {
                    self.score.score += rows * HARD_DROP_SCORE;
                    self.score.score_display.add(rows * HARD_DROP_SCORE);
                }
                self.events
                    .write(BoardEvent::HardDropped { from, to, color });
            }
        }
        log::info!("hard drop");
        self.lock(Sfx::HardDrop);
    }

    pub fn hold(&mut self) {
        if !self.mode.assists() || self.board.hold_used {
            return;
        }
        // no active rustomino between lock and spawn
        if self.board.playfield.ready_for_next() {
            return;
        }
        let next = match self.board.held.take() {
            Some(rustomino) => rustomino,
            None => self.get_next_rustomino(),
        };
        self.board.held = self.board.playfield.take_active();
        if !self.spawn(next.reset()) {
            log::info!("couldn't add held piece to board, collided with lock block");
            self.game_over();
        }
        // no more holds until the next lock
        self.board.hold_used = true;
        self.play(Sfx::Hold);
    }

    /// stop for the pause menu, dropping held controls
    pub fn pause(&mut self) {
        log::info!("game paused");
        self.controls.clear_inputs();
        self.next_screen.set(Screen::Paused);
    }

    pub fn game_over(&mut self) {
        log::info!("Game Over! Score: {}", self.score.score);
        self.board.topped_out = true;
        self.controls.clear_inputs();
        self.events.write(BoardEvent::ToppedOut {
            level: self.score.level,
        });
        self.next_screen.set(Screen::GameOver);
    }

    fn lock(&mut self, sound: Sfx) {
        let Some(rustomino) = &self.board.playfield.active_rustomino else {
            log::warn!("no active rustomino");
            return;
        };
        let (blocks, color) = (rustomino.playfield_slots(), rustomino.rtype.color());
        log::info!("locking block type: {:?}", rustomino.rtype);

        // a piece locked entirely above the playfield ends the game
        if fully_out_of_bounds(&blocks) {
            log::info!("block we are locking is fully out of playfield");
            self.game_over();
            return;
        }
        if sound == Sfx::Lock {
            self.events.write(BoardEvent::Locked { blocks, color });
        }
        self.board.hold_used = false;
        let Some(landed) = self.board.playfield.lock_active() else {
            return;
        };
        self.play(sound);
        self.handle_completed_lines(&landed);
    }

    /// moving or rotating during lockdown resets it
    fn increment_lockdown_resets(&mut self) {
        // landing while falling is handled in playing_update
        let Some(RustominoState::Lockdown { .. }) = self.board.playfield.get_active_state() else {
            return;
        };
        self.board.lockdown_resets += 1;
        let state = if self.board.playfield.active_can_fall() {
            RustominoState::Falling { time: 0. }
        } else {
            RustominoState::Lockdown { time: 0. }
        };
        self.board.playfield.set_active_state(state);
    }

    fn handle_completed_lines(&mut self, landed: &[IVec2]) {
        let lines = self
            .board
            .playfield
            .start_line_clear(LINE_CLEAR_DELAY, landed);
        if lines == 0 {
            return;
        }
        self.events.write(BoardEvent::LinesCleared { lines });
        let points = score_cleared_lines(lines, self.score.level);
        self.score.score += points;
        self.score.score_display.add(points);
        self.score.lines += lines;
        self.score.level_progress.add(lines);
        log::info!(
            "scored {points} for {lines} lines, {} lines in all",
            self.score.lines
        );

        // a level up replaces the clear sound so they don't overlap
        if self.score.lines >= self.score.level * LINES_PER_LEVEL {
            self.score.level += 1;
            log::info!("increasing game level to {}", self.score.level);
            self.board.gravity_delay = gravity_delay(self.score.level, *self.mode);
            self.events.write(BoardEvent::LevelUp);
            self.play(Sfx::LevelUp);
        } else {
            self.play(match lines {
                1 => Sfx::LineClear,
                2 => Sfx::LineClear2,
                3 => Sfx::LineClear3,
                _ => Sfx::Quad,
            });
        }
    }

    pub fn control(&mut self, control: Control) {
        match control {
            Control::Left => _ = self.translate(TranslationDirection::Left),
            Control::Right => _ = self.translate(TranslationDirection::Right),
            Control::RotateCW => _ = self.rotate(Rotation::Cw),
            Control::RotateCCW => _ = self.rotate(Rotation::Ccw),
            Control::SoftDrop => self.soft_drop(),
            Control::HardDrop => self.hard_drop(),
            Control::Hold => self.hold(),
        }
    }

    /// repeat held controls
    pub fn handle_held_inputs(&mut self, delta_time: f64) {
        for control in Control::iter() {
            if self.board.topped_out {
                return;
            }
            let new_state = match self.controls.input_states[&control] {
                InputState::Down(elapsed) => {
                    let elapsed = elapsed + delta_time;
                    match control.action_delay() {
                        Some(delay) if elapsed >= delay => {
                            self.control(control);
                            // carry over the remainder
                            InputState::Held(elapsed - delay)
                        }
                        Some(_) => InputState::Down(elapsed),
                        None => InputState::Up,
                    }
                }
                InputState::Held(elapsed) => {
                    let mut elapsed = elapsed + delta_time;
                    if let Some(delay) = control.action_repeat_delay() {
                        while elapsed >= delay {
                            self.control(control);
                            elapsed -= delay;
                        }
                    }
                    InputState::Held(elapsed)
                }
                InputState::Up => continue,
            };
            // handlers can clear inputs (pause, game over)
            if self.controls.input_states[&control] != InputState::Up {
                self.controls.input_states.insert(control, new_state);
            }
        }
    }

    /// one tick of play: clearing lines, or spawning and moving the active piece
    pub fn step(&mut self, delta_time: f64) {
        if self.board.topped_out {
            return;
        }
        self.handle_held_inputs(delta_time);
        if self.board.playfield.is_clearing_lines() {
            self.board.playfield.update_line_clear(delta_time);
        } else if !self.board.topped_out && self.ready_playfield() {
            self.playing_update(delta_time);
        }
    }
}

fn score_cleared_lines(num_lines: usize, level: usize) -> usize {
    level
        * match num_lines {
            1 => SINGLE_LINE_SCORE,
            2 => DOUBLE_LINE_SCORE,
            3 => TRIPLE_LINE_SCORE,
            4 => QUAD_SCORE,
            _ => panic!("impossible number of lines cleared"),
        }
}

/// all of the slots are above the playfield
fn fully_out_of_bounds(slots: &[IVec2; 4]) -> bool {
    slots.iter().all(|slot| slot[1] >= PLAYFIELD_SIZE[1])
}

/// gravity delay for the level (guideline), in seconds
pub fn gravity_delay(level: usize, mode: Mode) -> f64 {
    let level = match mode {
        Mode::Standard => level.min(STANDARD_TOP_LEVEL),
        Mode::Hard => level,
    };
    let n = level as i32 - 1;
    (0.8 - n as f64 * 0.007).powi(n).max(MIN_GRAVITY_DELAY)
}
