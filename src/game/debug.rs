use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use ggez::input::keyboard::KeyCode;

use super::{gravity_delay, BlocksState, GameState, Sfx, LINES_PER_LEVEL};
use crate::{
    background::{BackgroundStyle, FREEZE_TIME, LEVEL_UP_TIME, THAW_TIME},
    controls::Control,
    draw,
    effects::Effects,
    level_progress::LevelProgress,
    playfield::{Playfield, TranslationDirection, PLAYFIELD_SLOTS},
    rustomino::{Rotation, Rustomino, RustominoType},
    score_display::ScoreDisplay,
    scores::{Initials, ScoreBoard},
    style::{self, BlockStyle, Style},
};

const SPEEDS: [f64; 4] = [1.0, 0.5, 0.25, 0.1];
const FPS: [u32; 5] = [GAME_FPS, 144, 120, 60, 30]; // first is the game default
pub(super) const GAME_FPS: u32 = 240; // ticks during a benchmark
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
const STATS_WINDOW: Duration = Duration::from_millis(500); // frame stats averaged over
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
    pub replay: bool,     // loop with sound, or play once
    pub hidden: bool,     // hud off
    pub stats_only: bool, // just the fps panel, for a benchmark
    pub level: usize,     // board level, sets music speed
    pub view: GameState,  // screen drawn under the viewer
    time: f64,
    landed: Option<f64>, // when the falling piece locked
    step: f64,           // soft drop repeat
    surges: usize,       // background cues fired
    pub plays: usize,    // finished plays, a benchmark moves on after each
    ended: bool,         // this play is finished
    saved: Option<(ScoreBoard, Style, BackgroundStyle)>, // restored on close
    pub stats: FrameStats,
    pub recording: Option<Recording>, // frame samples going to a csv
}

/// one frame's drawing cost
#[derive(Clone, Copy)]
pub struct FrameSample {
    pub draws: usize,
    pub scene: Duration,    // building the draw list
    pub encode: Duration,   // ggez turning it into gpu commands
    pub gpu_wait: Duration, // blocked on the last frame's gpu work
}

/// frame samples averaged over a window, so the readout is steady
#[derive(Default)]
pub struct FrameStats {
    pub shown: Option<draw::FrameStats>,
    frames: u32,
    draws: usize,
    scene: Duration,
    encode: Duration,
    gpu_wait: Duration,
    elapsed: Duration,
}

impl FrameStats {
    pub fn record(&mut self, sample: FrameSample, delta: Duration) {
        self.frames += 1;
        self.draws += sample.draws;
        self.scene += sample.scene;
        self.encode += sample.encode;
        self.gpu_wait += sample.gpu_wait;
        self.elapsed += delta;
        if self.elapsed < STATS_WINDOW {
            return;
        }
        let ms = |total: Duration| total.as_secs_f32() * 1000.0 / self.frames as f32;
        self.shown = Some(draw::FrameStats {
            fps: self.frames as f32 / self.elapsed.as_secs_f32(),
            frame_ms: ms(self.elapsed),
            draws: self.draws / self.frames as usize,
            scene_ms: ms(self.scene),
            encode_ms: ms(self.encode),
            gpu_wait_ms: ms(self.gpu_wait),
        });
        *self = Self {
            shown: self.shown,
            ..Self::default()
        };
    }
}

// bump when the columns change, so old recordings aren't compared wrongly
const RECORDING_VERSION: u32 = 2;
const RECORDING_COLUMNS: &str = "frame,time_ms,frame_ms,animation,background,block,board,level,fps_cap,draws,scene_ms,encode_ms,gpu_wait_ms";

/// what a recording was made on, written at the top so runs can be compared
pub struct RunInfo {
    pub drawable: (f32, f32), // physical pixels
    pub scale: f32,           // physical pixels per view unit
    pub fps_cap: u32,         // 0 uncapped
    pub vsync: bool,
    pub kind: &'static str, // frames from C, or bench
}

/// where a frame was drawn, so rows of the same scene line up across runs
pub struct Scene {
    pub animation: &'static str,
    pub background: &'static str,
    pub block: &'static str,
    pub board: &'static str,
    pub level: usize,
    pub fps_cap: u32,
}

/// a recorded frame, formatted only when the recording is saved
pub(super) struct RecordedFrame {
    scene: Scene,
    pub sample: FrameSample,
    pub delta: Duration,
}

/// frame samples kept in memory, written to a csv when the recording stops
/// so no file io lands between the frames being measured
pub struct Recording {
    run: RunInfo,
    started: u64, // unix seconds, names the file
    frames: Vec<RecordedFrame>,
}

impl Recording {
    pub fn start(run: RunInfo) -> Self {
        let started = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // a minute at the cap up front, so growing rarely copies mid recording
        let frames = Vec::with_capacity(run.fps_cap.max(240) as usize * 60);
        Self {
            run,
            started,
            frames,
        }
    }

    pub fn record(&mut self, scene: Scene, sample: FrameSample, delta: Duration) {
        self.frames.push(RecordedFrame {
            scene,
            sample,
            delta,
        });
    }

    pub(super) fn len(&self) -> usize {
        self.frames.len()
    }

    pub(super) fn frames(&self) -> &[RecordedFrame] {
        &self.frames
    }

    /// write the csv, returns where
    pub fn finish(self) -> std::io::Result<PathBuf> {
        let file = format!("frame-stats/{}-{}.csv", self.run.kind, self.started);
        let path = crate::settings::data_path(&file)
            .ok_or_else(|| std::io::Error::other("no data directory"))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut out = BufWriter::new(File::create(&path)?);
        let run = &self.run;
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        writeln!(out, "# blocks frame stats v{RECORDING_VERSION}")?;
        writeln!(out, "# version={}", env!("CARGO_PKG_VERSION"))?;
        writeln!(out, "# commit={}", env!("BLOCKS_COMMIT"))?;
        writeln!(out, "# profile={profile}")?;
        writeln!(
            out,
            "# os={}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )?;
        writeln!(
            out,
            "# drawable={}x{} scale={}",
            run.drawable.0, run.drawable.1, run.scale
        )?;
        writeln!(out, "# fps_cap={} vsync={}", run.fps_cap, run.vsync)?;
        writeln!(out, "# kind={}", run.kind)?;
        writeln!(out, "{RECORDING_COLUMNS}")?;
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let mut time = Duration::ZERO;
        for (i, frame) in self.frames.iter().enumerate() {
            let (scene, sample) = (&frame.scene, &frame.sample);
            time += frame.delta;
            writeln!(
                out,
                "{},{:.3},{:.3},{},{},{},{},{},{},{},{:.4},{:.4},{:.4}",
                i,
                ms(time),
                ms(frame.delta),
                scene.animation,
                scene.background,
                scene.block,
                scene.board,
                scene.level,
                scene.fps_cap,
                sample.draws,
                ms(sample.scene),
                ms(sample.encode),
                ms(sample.gpu_wait),
            )?;
        }
        out.flush()?;
        Ok(path)
    }
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
            stats_only: false,
            level: 4,
            view: GameState::Playing,
            time: 0.0,
            landed: None,
            step: 0.0,
            surges: 0,
            plays: 0,
            ended: false,
            saved: None,
            stats: FrameStats::default(),
            recording: None,
        }
    }

    pub fn animation(&self) -> Animation {
        Animation::ALL[self.selected]
    }

    pub fn speed(&self) -> f64 {
        SPEEDS[self.speed]
    }

    pub(super) fn speed_index(&self) -> usize {
        self.speed
    }

    pub(super) fn set_speed_index(&mut self, speed: usize) {
        self.speed = speed.min(SPEEDS.len() - 1);
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
        // no stale numbers from the last visit
        self.debug.stats = FrameStats::default();
        self.set_state(GameState::Debug);
        self.start_animation();
    }

    /// start or stop writing frame samples to a csv
    fn toggle_recording(&mut self, run: RunInfo) {
        match self.debug.recording.take() {
            Some(recording) => self.stop_recording(recording),
            None => self.debug.recording = Some(Recording::start(run)),
        }
    }

    pub(super) fn stop_recording(&mut self, recording: Recording) {
        match recording.finish() {
            Ok(path) => println!("frame stats saved to {}", path.display()),
            Err(e) => log::error!("could not save the frame stats recording: {e}"),
        }
    }

    /// row for the recording, if one is running
    pub(super) fn record_frame(&mut self, sample: FrameSample, delta: Duration) {
        let scene = Scene {
            animation: self.debug.animation().name(),
            background: self.background.style.name(),
            block: self.style.block.name(),
            board: self.style.board.name(),
            level: self.debug.level,
            fps_cap: if self.bench.is_some() {
                0
            } else {
                self.debug.fps()
            },
        };
        if let Some(recording) = self.debug.recording.as_mut() {
            recording.record(scene, sample, delta);
        }
    }

    fn close_debug(&mut self) {
        // Esc ends a benchmark early
        self.finish_bench(false);
        if let Some(recording) = self.debug.recording.take() {
            self.stop_recording(recording);
        }
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
        self.debug.ended = false;
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
        if !self.debug.ended && self.debug.time >= length + LOOP_GAP {
            self.debug.ended = true;
            self.debug.plays += 1;
            if self.debug.replay {
                self.start_animation();
            }
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
            KeyCode::KeyC if !repeated && self.bench.is_none() => {
                let view = &self.view_settings;
                let run = RunInfo {
                    drawable: (view.view_rect.w * view.scale, view.view_rect.h * view.scale),
                    scale: view.scale,
                    fps_cap: self.debug.fps(),
                    vsync: self.vsync,
                    kind: "frames",
                };
                self.toggle_recording(run);
            }
            KeyCode::KeyF if !repeated => {
                let count = FPS.len();
                self.debug.fps = if shift {
                    (self.debug.fps + count - 1) % count
                } else {
                    (self.debug.fps + 1) % count
                };
            }
            KeyCode::KeyT if !repeated && self.bench.is_none() => {
                self.start_bench(super::bench::StartBench {
                    hold: super::bench::DEFAULT_HOLD,
                    filter: None,
                    exit: false,
                    vsync: self.vsync,
                });
            }
            KeyCode::KeyB => self.background.cycle(1),
            KeyCode::KeyS => self.style.block = style::cycle(&BlockStyle::ALL, self.style.block, 1),
            _ => {}
        }
    }
}
