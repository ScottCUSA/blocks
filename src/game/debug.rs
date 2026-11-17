use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use super::board::{gravity_delay, Board, Mode, Play, Score};
use super::input::{HandleKeys, Keys};
use super::screenshot::{fill_rows, setup_board, setup_sample_scores, setup_tall_stack};
use super::{sim_dt, ScoreView, Screen, Sim, Viewer, LINES_PER_LEVEL};
use crate::{
    audio::{MusicCue, PlaySfx, Sfx},
    background::{Background, BackgroundStyle, FREEZE_TIME, LEVEL_UP_TIME, THAW_TIME},
    controls::Control,
    draw::{self, ViewSettings},
    effects::{reset_effects, BoardFx},
    level_progress::LevelProgress,
    playfield::{TranslationDirection, PLAYFIELD_SLOTS},
    rustomino::{Rotation, Rustomino, RustominoType},
    score_display::ScoreDisplay,
    scores::{Initials, ScoreBoard},
    settings::Settings,
    style::{self, BlockStyle, Style},
};

const SPEEDS: [f64; 4] = [1.0, 0.5, 0.25, 0.1];
// frame caps, the viewer starts unlocked
const FPS: [Option<u32>; 6] = [None, Some(240), Some(144), Some(120), Some(60), Some(30)];
const GAME_FPS: u32 = 240; // ticks, and the game's cap when the viewer is unlocked
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
    pub(super) fn length(self) -> f64 {
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
#[derive(Resource)]
pub struct DebugView {
    pub selected: usize,
    speed: usize,
    fps: usize, // kept after closing, for the session
    pub paused: bool,
    pub replay: bool,     // loop with sound, or play once
    pub hidden: bool,     // hud off
    pub stats_only: bool, // just the fps panel, for a benchmark
    pub level: usize,     // board level, sets music speed
    pub view: Screen,     // screen drawn under the viewer
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
    pub draws: usize, // vertex runs, one gpu draw each
    pub vertices: usize,
    pub scene: Duration, // building the draw list
}

/// bevy's smoothed frame timing
#[derive(Clone, Copy, Default)]
pub struct FrameTiming {
    pub fps: f64,
    pub frame_ms: f64,
}

/// frame samples averaged over a window, so the readout is steady
#[derive(Default)]
pub struct FrameStats {
    pub shown: Option<draw::FrameStats>,
    frames: u32,
    draws: usize,
    vertices: usize,
    scene: Duration,
    elapsed: Duration,
}

impl FrameStats {
    pub fn record(&mut self, sample: FrameSample, delta: Duration, timing: FrameTiming) {
        self.frames += 1;
        self.draws += sample.draws;
        self.vertices += sample.vertices;
        self.scene += sample.scene;
        self.elapsed += delta;
        if self.elapsed < STATS_WINDOW {
            return;
        }
        let frames = self.frames as usize;
        self.shown = Some(draw::FrameStats {
            draws: self.draws / frames,
            vertices: self.vertices / frames,
            scene_ms: self.scene.as_secs_f32() * 1000.0 / self.frames as f32,
            fps: timing.fps as f32,
            frame_ms: timing.frame_ms as f32,
        });
        *self = Self {
            shown: self.shown,
            ..Self::default()
        };
    }
}

// bump when the columns change, so old recordings aren't compared wrongly
const RECORDING_VERSION: u32 = 2;
const RECORDING_COLUMNS: &str =
    "frame,time_ms,frame_ms,animation,background,block,board,level,fps_cap,draws,vertices,scene_ms";

/// what a recording was made on, written at the top so runs can be compared
pub struct RunInfo {
    pub drawable: (f32, f32), // physical pixels
    pub scale: f32,           // physical pixels per view unit
    pub fps_cap: u32,
    pub kind: &'static str, // names the file
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
        let frames = Vec::with_capacity(run.fps_cap.max(GAME_FPS) as usize * 60);
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
        writeln!(out, "# fps_cap={}", run.fps_cap)?;
        writeln!(out, "{RECORDING_COLUMNS}")?;
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let mut time = Duration::ZERO;
        for (i, frame) in self.frames.iter().enumerate() {
            let (scene, sample) = (&frame.scene, &frame.sample);
            time += frame.delta;
            writeln!(
                out,
                "{},{:.3},{:.3},{},{},{},{},{},{},{},{},{:.4}",
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
                sample.vertices,
                ms(sample.scene),
            )?;
        }
        out.flush()?;
        Ok(path)
    }
}

impl DebugView {
    /// the player's style and background while the viewer shows others
    pub fn saved_style(&self) -> Option<(Style, BackgroundStyle)> {
        self.saved
            .as_ref()
            .map(|(_, style, background)| (*style, *background))
    }

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
            view: Screen::Playing,
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

    /// the viewer's frame cap, None unlocked
    pub fn fps_cap(&self) -> Option<u32> {
        FPS[self.fps]
    }

    /// game ticks per second, and the game's frame cap
    pub fn tick_rate(&self) -> u32 {
        self.fps_cap().unwrap_or(GAME_FPS)
    }
}

impl DebugView {
    /// row for the recording, and the averaged readout
    pub fn record_frame(
        &mut self,
        scene: Scene,
        sample: FrameSample,
        delta: Duration,
        timing: FrameTiming,
    ) {
        self.stats.record(sample, delta, timing);
        if let Some(recording) = self.recording.as_mut() {
            recording.record(scene, sample, delta);
        }
    }

    /// start or stop writing frame samples to a csv
    fn toggle_recording(&mut self, run: RunInfo) {
        match self.recording.take() {
            Some(recording) => stop_recording(recording),
            None => self.recording = Some(Recording::start(run)),
        }
    }
}

pub(super) fn stop_recording(recording: Recording) {
    match recording.finish() {
        Ok(path) => println!("frame stats saved to {}", path.display()),
        Err(e) => log::error!("could not save the frame stats recording: {e}"),
    }
}

/// open the animation viewer
#[derive(Message, Debug, Clone, Copy)]
pub struct OpenViewer;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OpenViewer>()
            .add_systems(
                bevy::app::RunFixedMainLoop,
                (
                    open_viewer.run_if(on_message::<OpenViewer>),
                    keys.run_if(in_state(Viewer::On)),
                )
                    .in_set(HandleKeys)
                    .after(super::input::collect_keys),
            )
            .add_systems(
                FixedUpdate,
                sim_dt
                    .pipe(step)
                    .in_set(Sim::Play)
                    .run_if(in_state(Viewer::On)),
            );
    }
}

pub(super) fn open_viewer(world: &mut World) {
    let scores = std::mem::take(&mut *world.resource_mut::<ScoreBoard>());
    let style = *world.resource::<Style>();
    let background = world.resource::<Background>().style;
    let mut debug = world.resource_mut::<DebugView>();
    debug.saved = Some((scores, style, background));
    // no stale numbers from the last visit
    debug.stats = FrameStats::default();
    world.resource_mut::<NextState<Viewer>>().set(Viewer::On);
    start_animation(world);
}

fn close_viewer(world: &mut World) {
    let mut debug = world.resource_mut::<DebugView>();
    if let Some(recording) = debug.recording.take() {
        stop_recording(recording);
    }
    if let Some((scores, style, background)) = debug.saved.take() {
        *world.resource_mut::<ScoreBoard>() = scores;
        *world.resource_mut::<Style>() = style;
        world.resource_mut::<Background>().style = background;
    }
    let initials = Initials::new(&world.resource::<Settings>().initials);
    *world.resource_mut::<Initials>() = initials;
    world.resource_mut::<ScoreView>().highlight = None;
    world.write_message(PlaySfx(Sfx::MenuSelect));
    world.resource_mut::<NextState<Viewer>>().set(Viewer::Off);
    // entering the menu starts a new game
    world.resource_mut::<NextState<Screen>>().set(Screen::Menu);
}

/// run game actions on the world
pub(super) fn with_play<R>(world: &mut World, f: impl FnOnce(&mut Play) -> R) -> R {
    let mut state = SystemState::<Play>::new(world);
    let mut play = state
        .get_mut(world)
        .expect("game resources are always present");
    let result = f(&mut play);
    state.apply(world);
    result
}

/// reset the board and start the selected animation
pub(super) fn start_animation(world: &mut World) {
    world.write_message(MusicCue::EndGameOver);
    let (animation, level) = {
        let mut debug = world.resource_mut::<DebugView>();
        debug.time = 0.0;
        debug.landed = None;
        debug.step = 0.0;
        debug.surges = 0;
        debug.ended = false;
        debug.view = Screen::Playing;
        (debug.animation(), debug.level)
    };
    let mode = *world.resource::<Mode>();
    *world.resource_mut::<Board>() = Board::new(mode);
    let _ = world.run_system_cached(reset_effects);
    *world.resource_mut::<ScoreBoard>() = ScoreBoard::default();
    world.resource_mut::<ScoreView>().highlight = None;
    world.resource_mut::<Background>().reset();
    setup_board(world);
    {
        let mut score = world.resource_mut::<Score>();
        score.level = level;
        score.level_display = ScoreDisplay::new(level);
    }
    world.resource_mut::<Board>().gravity_delay = gravity_delay(level, mode);
    let mut view = Screen::Playing;
    match animation {
        Animation::Single => debug_clear(world, 1, 2),
        Animation::Double => debug_clear(world, 2, 2),
        Animation::Triple => debug_clear(world, 3, 2),
        Animation::Quad => debug_clear(world, 4, 2),
        Animation::LevelUp => debug_clear(world, 2, LINES_PER_LEVEL - 2),
        Animation::HardDrop => with_play(world, |play| {
            play.spawn(Rustomino::new(RustominoType::T));
            play.hard_drop();
        }),
        Animation::Gravity | Animation::SoftDrop => with_play(world, |play| {
            play.spawn(Rustomino::new(RustominoType::T));
        }),
        Animation::Lock => with_play(world, |play| {
            play.spawn(Rustomino::new(RustominoType::T));
            let playfield = &mut play.board.playfield;
            let rows = match (&playfield.active_rustomino, &playfield.ghost_rustomino) {
                (Some(active), Some(ghost)) => {
                    active.playfield_slots()[0].y - ghost.playfield_slots()[0].y
                }
                _ => 0,
            };
            for _ in LOCK_ROWS..rows {
                playfield.translate_active(TranslationDirection::Down);
            }
        }),
        Animation::GameOver => {
            setup_tall_stack(world);
            world.resource_mut::<BoardFx>().game_over();
            world.resource_mut::<Background>().freeze(level);
            world.write_message(MusicCue::GameOver);
            view = Screen::GameOver;
        }
        Animation::NewHighScore => {
            world.write_message(PlaySfx(Sfx::HighScore));
            setup_tall_stack(world);
            let mut fx = world.resource_mut::<BoardFx>();
            fx.game_over();
            fx.set_game_over_time(4.0);
            let mut background = world.resource_mut::<Background>();
            background.freeze(level);
            background.skip(4.0, level);
            let mut initials = Initials::new("SCO");
            initials.cursor = 1;
            *world.resource_mut::<Initials>() = initials;
            view = Screen::NameEntry;
        }
        Animation::HighScores => {
            setup_sample_scores(world);
            world.resource_mut::<ScoreView>().highlight = Some(2);
            view = Screen::HighScores;
        }
        // background alone
        Animation::Background => view = Screen::Backdrop,
    }
    world.resource_mut::<DebugView>().view = view;
    world.resource_mut::<NextState<Screen>>().set(view);
}

/// clear `lines` rows with an I dropped down the center
fn debug_clear(world: &mut World, lines: usize, filled: usize) {
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
    {
        let mut board = world.resource_mut::<Board>();
        board.playfield = crate::playfield::Playfield::new();
        fill_rows(&mut board, 0, &rows);
    }
    {
        let mut score = world.resource_mut::<Score>();
        score.level_progress = LevelProgress::new(LINES_PER_LEVEL, filled);
        score.lines = (score.level - 1) * LINES_PER_LEVEL + filled;
    }
    with_play(world, |play| {
        play.spawn(Rustomino::new(RustominoType::I));
        play.board.playfield.rotate_active(Rotation::Cw);
        while let Some(x) = play
            .board
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
            if !play.board.playfield.translate_active(direction) {
                break;
            }
        }
        play.hard_drop();
    });
}

/// the viewer's part of a tick: background cues, the falling piece and replays
fn step(
    In(dt): In<f64>,
    mut commands: Commands,
    mut debug: ResMut<DebugView>,
    mut background: ResMut<Background>,
    mut play: Play,
) {
    // paused or unfocused
    if dt == 0.0 {
        return;
    }
    debug.time += dt;
    play.board.playfield.update_line_clear(dt);
    if debug.animation() == Animation::Background {
        while debug.surges < SURGES + 3 && debug.time >= background_cue(debug.surges + 1) {
            debug.surges += 1;
            match debug.surges.saturating_sub(SURGES) {
                0 => background.surge(debug.surges),
                1 => background.level_up(),
                2 => background.freeze(play.score.level),
                3 => background.thaw(),
                _ => {}
            }
        }
    }
    let mut length = debug.animation().length();
    if debug.animation().falls() {
        if play.board.playfield.active_rustomino.is_some() {
            if debug.animation() == Animation::SoftDrop {
                // soft drop at the held key repeat rate
                if let Some(repeat) = Control::SoftDrop.action_repeat_delay() {
                    debug.step += dt;
                    while debug.step >= repeat && play.board.playfield.active_can_fall() {
                        debug.step -= repeat;
                        play.soft_drop();
                    }
                }
            }
            play.playing_update(dt);
        } else if debug.landed.is_none() {
            debug.landed = Some(debug.time);
        }
        length = debug.landed.map_or(length, |t| t + LANDED_GAP);
    }
    if !debug.ended && debug.time >= length + LOOP_GAP {
        debug.ended = true;
        debug.plays += 1;
        if debug.replay {
            commands.run_system_cached(start_animation);
        }
    }
}

fn keys(
    keys: Res<Keys>,
    mut commands: Commands,
    mut debug: ResMut<DebugView>,
    mut background: ResMut<Background>,
    mut style: ResMut<Style>,
    view: Res<ViewSettings>,
    mut sfx: MessageWriter<PlaySfx>,
    bench: Option<Res<super::bench::Bench>>,
) {
    let bench_running = bench.is_some();
    let count = Animation::ALL.len();
    let step = if keys.shift { LEVEL_STEP } else { 1 };
    let mut restart = false;
    for press in &keys.pressed {
        let (keycode, repeated) = (press.key, press.repeat);
        match keycode {
            KeyCode::Escape if !repeated => {
                commands.run_system_cached(close_viewer);
                return;
            }
            KeyCode::ArrowUp | KeyCode::ArrowDown => {
                debug.selected = if keycode == KeyCode::ArrowUp {
                    (debug.selected + count - 1) % count
                } else {
                    (debug.selected + 1) % count
                };
                sfx.write(PlaySfx(Sfx::MenuMove));
                restart = true;
            }
            KeyCode::ArrowLeft if debug.speed + 1 < SPEEDS.len() => debug.speed += 1,
            KeyCode::ArrowRight if debug.speed > 0 => debug.speed -= 1,
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space if !repeated => {
                debug.paused = false;
                restart = true;
            }
            KeyCode::Minus | KeyCode::NumpadSubtract if debug.level > 1 => {
                debug.level = debug.level.saturating_sub(step).max(1);
                restart = true;
            }
            KeyCode::Equal | KeyCode::NumpadAdd if debug.level < MAX_LEVEL => {
                debug.level = (debug.level + step).min(MAX_LEVEL);
                restart = true;
            }
            KeyCode::KeyP if !repeated => debug.paused = !debug.paused,
            KeyCode::KeyR if !repeated => debug.replay = !debug.replay,
            KeyCode::KeyH if !repeated => debug.hidden = !debug.hidden,
            KeyCode::KeyC if !repeated => {
                let run = RunInfo {
                    drawable: (view.view_rect.w * view.scale, view.view_rect.h * view.scale),
                    scale: view.scale,
                    fps_cap: debug.fps_cap().unwrap_or(0),
                    kind: "frames",
                };
                debug.toggle_recording(run);
            }
            KeyCode::KeyF if !repeated => {
                let count = FPS.len();
                debug.fps = if keys.shift {
                    (debug.fps + count - 1) % count
                } else {
                    (debug.fps + 1) % count
                };
            }
            KeyCode::KeyT if !repeated && !bench_running => {
                commands.insert_resource(super::bench::StartBench {
                    hold: super::bench::DEFAULT_HOLD,
                    filter: None,
                    exit: false,
                });
            }
            KeyCode::KeyB => background.cycle(1),
            KeyCode::KeyS => style.block = style::cycle(&BlockStyle::ALL, style.block, 1),
            _ => {}
        }
    }
    if restart {
        commands.run_system_cached(start_animation);
    }
}
