use std::cmp::Ordering;

use ggez::input::keyboard::KeyCode;

use super::{gravity_delay, BlocksState, GameState, Sfx, LINES_PER_LEVEL};
use crate::{
    background::{BackgroundStyle, FREEZE_TIME, LEVEL_UP_TIME, THAW_TIME},
    controls::Control,
    effects::Effects,
    level_progress::LevelProgress,
    playfield::{Playfield, TranslationDirection, PLAYFIELD_SLOTS},
    rustomino::{Rotation, Rustomino, RustominoType},
    score_display::ScoreDisplay,
    scores::{Initials, ScoreBoard},
    style::{self, BlockStyle, Style},
};

const SPEEDS: [f64; 4] = [1.0, 0.5, 0.25, 0.1];
const FPS: [u32; 5] = [240, 144, 120, 60, 30]; // first is the game default
const LOOP_GAP: f64 = 0.6; // seconds before replaying
const MAX_LEVEL: usize = 255;
const LEVEL_STEP: usize = 10; // with shift
const LOCK_ROWS: i32 = 2; // falls before the lock
const LANDED_GAP: f64 = 0.4; // seconds after a gravity drop locks
const SURGE_GAP: f64 = 1.2; // seconds between background surges
const SURGES: usize = 4; // one per line clear size, then a level up
const FREEZE_GAP: f64 = 0.7; // after the level up
const FROZEN_HOLD: f64 = 1.5; // stopped before the thaw
const THAW_GAP: f64 = 0.5; // after the thaw
/// time of background cue i, from 1
fn background_cue(i: usize) -> f64 {
    let level_up = (SURGES + 1) as f64 * SURGE_GAP;
    let freeze = level_up + LEVEL_UP_TIME as f64 + FREEZE_GAP;
    match i.saturating_sub(SURGES) {
        0 | 1 => i as f64 * SURGE_GAP,
        2 => freeze,
        _ => freeze + FREEZE_TIME as f64 + FROZEN_HOLD,
    }
}

const WELL: i32 = PLAYFIELD_SLOTS[0] as i32 / 2 - 1;
// bottom row first, the center column is left open for an I
const CLEAR_ROWS: [&str; 4] = ["LLZZ.OOSSJ", "LZZT.OOSJJ", "TTJJ.LLOOZ", "TZZS.SJJOO"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Animation {
    Single,
    Double,
    Triple,
    Quad,
    LevelUp,
    HardDrop,
    Gravity,
    SoftDrop,
    Lock,
    GameOver,
    NewHighScore,
    HighScores,
    Background,
}

impl Animation {
    pub const ALL: [Animation; 13] = [
        Animation::Gravity,
        Animation::SoftDrop,
        Animation::HardDrop,
        Animation::Lock,
        Animation::Single,
        Animation::Double,
        Animation::Triple,
        Animation::Quad,
        Animation::LevelUp,
        Animation::GameOver,
        Animation::NewHighScore,
        Animation::HighScores,
        Animation::Background,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Animation::Single => "Single",
            Animation::Double => "Double",
            Animation::Triple => "Triple",
            Animation::Quad => "Quad",
            Animation::LevelUp => "Level Up",
            Animation::HardDrop => "Hard Drop",
            Animation::Gravity => "Gravity",
            Animation::SoftDrop => "Soft Drop",
            Animation::Lock => "Lock",
            Animation::GameOver => "Game Over",
            Animation::NewHighScore => "New High Score",
            Animation::HighScores => "High Scores",
            Animation::Background => "Background",
        }
    }

    /// seconds before replaying
    fn length(self) -> f64 {
        match self {
            Animation::Single | Animation::Double | Animation::Triple | Animation::Quad => 1.6,
            Animation::LevelUp => 2.2,
            Animation::HardDrop => 1.0,
            // until the piece locks
            Animation::Gravity | Animation::SoftDrop | Animation::Lock => f64::INFINITY,
            Animation::GameOver => 4.5,
            Animation::NewHighScore => 1.6,
            Animation::HighScores => 2.0,
            // calm, a surge per clear size, level up, game over freeze, thaw
            Animation::Background => background_cue(SURGES + 3) + THAW_TIME as f64 + THAW_GAP,
        }
    }

    /// piece played by the game until it locks
    fn falls(self) -> bool {
        matches!(
            self,
            Animation::Gravity | Animation::SoftDrop | Animation::Lock
        )
    }
}

/// animation viewer state
pub struct DebugView {
    pub selected: usize,
    speed: usize,
    fps: usize, // kept after closing, for the session
    pub paused: bool,
    pub replay: bool,    // loop with sound, or play once
    pub hidden: bool,    // hud off
    pub level: usize,    // board level, sets music speed
    pub view: GameState, // screen drawn under the viewer
    time: f64,
    landed: Option<f64>, // when the falling piece locked
    step: f64,           // soft drop repeat
    surges: usize,       // background cues fired
    saved: Option<(ScoreBoard, Style, BackgroundStyle)>, // restored on close
}

impl DebugView {
    pub fn new() -> Self {
        Self {
            selected: 0,
            speed: 0,
            fps: 0,
            paused: false,
            replay: true,
            hidden: false,
            level: 4,
            view: GameState::Playing,
            time: 0.0,
            landed: None,
            step: 0.0,
            surges: 0,
            saved: None,
        }
    }

    pub fn animation(&self) -> Animation {
        Animation::ALL[self.selected]
    }

    pub fn speed(&self) -> f64 {
        SPEEDS[self.speed]
    }

    /// game ticks and drawn frames per second
    pub fn fps(&self) -> u32 {
        FPS[self.fps]
    }
}

impl BlocksState {
    pub(super) fn open_debug(&mut self) {
        let scores = std::mem::take(&mut self.scores);
        self.debug.saved = Some((scores, self.style, self.background.style));
        self.set_state(GameState::Debug);
        self.start_animation();
    }

    fn close_debug(&mut self) {
        if let Some((scores, style, background)) = self.debug.saved.take() {
            self.scores = scores;
            self.style = style;
            self.background.style = background;
        }
        self.initials = Initials::new(&self.settings.initials);
        self.highlight = None;
        self.background.reset();
        self.assets.play(Sfx::MenuSelect);
        self.new_game();
    }

    /// reset the board and start the selected animation
    pub(super) fn start_animation(&mut self) {
        self.assets.end_game_over();
        self.debug.time = 0.0;
        self.debug.landed = None;
        self.debug.step = 0.0;
        self.debug.surges = 0;
        self.playfield = Playfield::new();
        self.effects = Effects::new();
        self.scores = ScoreBoard::default();
        self.highlight = None;
        self.hold_used = false;
        self.background.reset();
        self.setup_board();
        self.level = self.debug.level;
        self.level_display = ScoreDisplay::new(self.level);
        self.gravity_delay = gravity_delay(self.level, self.mode);
        self.lockdown_resets = 0;
        self.debug.view = GameState::Playing;
        match self.debug.animation() {
            Animation::Single => self.debug_clear(1, 2),
            Animation::Double => self.debug_clear(2, 2),
            Animation::Triple => self.debug_clear(3, 2),
            Animation::Quad => self.debug_clear(4, 2),
            Animation::LevelUp => self.debug_clear(2, LINES_PER_LEVEL - 2),
            Animation::HardDrop => {
                self.spawn(Rustomino::new(RustominoType::T));
                self.playfield.translate_active(TranslationDirection::Right);
                self.hard_drop();
            }
            Animation::Gravity | Animation::SoftDrop => {
                self.spawn(Rustomino::new(RustominoType::T));
            }
            Animation::Lock => {
                self.spawn(Rustomino::new(RustominoType::T));
                self.playfield.translate_active(TranslationDirection::Right);
                let rows = match (
                    &self.playfield.active_rustomino,
                    &self.playfield.ghost_rustomino,
                ) {
                    (Some(active), Some(ghost)) => {
                        active.playfield_slots()[0].y - ghost.playfield_slots()[0].y
                    }
                    _ => 0,
                };
                for _ in LOCK_ROWS..rows {
                    self.playfield.translate_active(TranslationDirection::Down);
                }
            }
            Animation::GameOver => {
                self.setup_tall_stack();
                self.effects.game_over();
                self.background.freeze(self.level);
                if self.screenshot.is_none() {
                    self.assets.play_game_over();
                }
                self.debug.view = GameState::GameOver;
            }
            Animation::NewHighScore => {
                self.assets.play(Sfx::HighScore);
                self.setup_tall_stack();
                self.effects.game_over();
                self.effects.update(4.0);
                self.background.freeze(self.level);
                self.background.skip(4.0, self.level);
                self.initials = Initials::new("SCO");
                self.initials.cursor = 1;
                self.debug.view = GameState::NameEntry;
            }
            Animation::HighScores => {
                self.setup_sample_scores();
                self.highlight = Some(2);
                self.debug.view = GameState::HighScores;
            }
            // background alone
            Animation::Background => self.debug.view = GameState::Debug,
        }
    }

    /// clear `lines` rows with an I dropped down the center
    fn debug_clear(&mut self, lines: usize, filled: usize) {
        self.playfield = Playfield::new();
        let rows: Vec<String> = CLEAR_ROWS
            .iter()
            .enumerate()
            .map(|(y, row)| {
                let mut row = row.to_string();
                if y >= lines {
                    row.replace_range(y + 6..y + 7, ".");
                }
                row
            })
            .collect();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        self.fill_rows(0, &rows);
        self.level_progress = LevelProgress::new(LINES_PER_LEVEL, filled);
        self.total_lines_cleared = (self.level - 1) * LINES_PER_LEVEL + filled;

        self.spawn(Rustomino::new(RustominoType::I));
        self.playfield.rotate_active(Rotation::Cw);
        while let Some(x) = self
            .playfield
            .active_rustomino
            .as_ref()
            .map(|r| r.playfield_slots()[0].x)
        {
            let direction = match x.cmp(&WELL) {
                Ordering::Less => TranslationDirection::Right,
                Ordering::Greater => TranslationDirection::Left,
                Ordering::Equal => break,
            };
            if !self.playfield.translate_active(direction) {
                break;
            }
        }
        self.hard_drop();
    }

    pub(super) fn debug_update(&mut self, delta_time: f64) {
        // replays restart audio, hold while unfocused
        if self.debug.paused || !self.focused {
            return;
        }
        let dt = delta_time * self.debug.speed();
        self.debug.time += dt;
        self.score_display.update(dt);
        self.level_display.update(dt);
        if self.level_progress.update(dt) {
            self.level_display.add(1);
        }
        self.effects.update(dt);
        self.background.update(dt, self.level);
        self.playfield.update_line_clear(dt);
        if self.debug.animation() == Animation::Background {
            while self.debug.surges < SURGES + 3
                && self.debug.time >= background_cue(self.debug.surges + 1)
            {
                self.debug.surges += 1;
                match self.debug.surges.saturating_sub(SURGES) {
                    0 => self.background.surge(self.debug.surges),
                    1 => self.background.level_up(),
                    2 => self.background.freeze(self.level),
                    3 => self.background.thaw(),
                    _ => {}
                }
            }
        }
        let mut length = self.debug.animation().length();
        if self.debug.animation().falls() {
            if self.playfield.active_rustomino.is_some() {
                if self.debug.animation() == Animation::SoftDrop {
                    self.debug_soft_drop(dt);
                }
                self.playing_update(dt);
            } else if self.debug.landed.is_none() {
                self.debug.landed = Some(self.debug.time);
            }
            length = self.debug.landed.map_or(length, |t| t + LANDED_GAP);
        }
        if self.debug.replay && self.debug.time >= length + LOOP_GAP {
            self.start_animation();
        }
    }

    /// soft drop at the held key repeat rate
    fn debug_soft_drop(&mut self, dt: f64) {
        let Some(repeat) = Control::SoftDrop.action_repeat_delay() else {
            return;
        };
        self.debug.step += dt;
        while self.debug.step >= repeat && self.playfield.active_can_fall() {
            self.debug.step -= repeat;
            self.soft_drop();
        }
    }

    pub(super) fn debug_key(&mut self, keycode: KeyCode, repeated: bool, shift: bool) {
        let count = Animation::ALL.len();
        let step = if shift { LEVEL_STEP } else { 1 };
        match keycode {
            KeyCode::Escape if !repeated => self.close_debug(),
            KeyCode::ArrowUp | KeyCode::ArrowDown => {
                self.debug.selected = if keycode == KeyCode::ArrowUp {
                    (self.debug.selected + count - 1) % count
                } else {
                    (self.debug.selected + 1) % count
                };
                self.assets.play(Sfx::MenuMove);
                self.start_animation();
            }
            KeyCode::ArrowLeft if self.debug.speed + 1 < SPEEDS.len() => self.debug.speed += 1,
            KeyCode::ArrowRight if self.debug.speed > 0 => self.debug.speed -= 1,
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space if !repeated => {
                self.debug.paused = false;
                self.start_animation();
            }
            KeyCode::Minus | KeyCode::NumpadSubtract if self.debug.level > 1 => {
                self.debug.level = self.debug.level.saturating_sub(step).max(1);
                self.start_animation();
            }
            KeyCode::Equal | KeyCode::NumpadAdd if self.debug.level < MAX_LEVEL => {
                self.debug.level = (self.debug.level + step).min(MAX_LEVEL);
                self.start_animation();
            }
            KeyCode::KeyP if !repeated => self.debug.paused = !self.debug.paused,
            KeyCode::KeyR if !repeated => self.debug.replay = !self.debug.replay,
            KeyCode::KeyH if !repeated => self.debug.hidden = !self.debug.hidden,
            KeyCode::KeyF if !repeated => {
                let count = FPS.len();
                self.debug.fps = if shift {
                    (self.debug.fps + count - 1) % count
                } else {
                    (self.debug.fps + 1) % count
                };
            }
            KeyCode::KeyB => self.background.cycle(1),
            KeyCode::KeyS => self.style.block = style::cycle(&BlockStyle::ALL, self.style.block, 1),
            _ => {}
        }
    }
}
