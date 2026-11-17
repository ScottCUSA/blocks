use std::{fs, io::BufWriter, path::Path, path::PathBuf};

use crate::{GameError, GameResult};

use bevy::prelude::*;

use super::board::{Board, BoardEvent, Score, LINE_CLEAR_DELAY};
use super::{OptionsFrom, ScoreView, Screen, LINES_PER_LEVEL};
use crate::background::{Background, BackgroundStyle};
use crate::effects::{on_board_events, step_effects, BoardFx};
use crate::level_progress::LevelProgress;
use crate::menus::{Menu, Menus};
use crate::score_display::ScoreDisplay;
use crate::scores::{HighScore, Initials, ScoreBoard};
use crate::style::{BlockStyle, BoardStyle, Style};
use crate::{
    playfield::{LineClear, SlotState, TranslationDirection},
    rustomino::{Rustomino, RustominoType},
};

// bottom row first, '.' is empty
const STACK: [&str; 5] = [
    "LLZ.OOSSJI",
    "LZZ.OOSJJI",
    "LZT..TSJ.I",
    ".TT...S..I",
    "......O...",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scene {
    Menu,
    Playing,
    LineClear,
    QuadClear,
    HardDrop,
    Stars,
    Plasma,
    Storm,
    StormMax,
    StormWhiteout,
    Paused,
    GameOver,
    GameOverSweep,
    GameOverDrop,
    Options,
    OptionsPaused,
    OptionsAudio,
    Score,
    Flat,
    Neon,
    Retro,
    Jelly,
    NameEntry,
    HighScores,
    HighScoresNew,
    ProgressFill,
    LevelUp,
    Bench,
    Demo,
}

impl Scene {
    const ALL: [(&'static str, Scene); 29] = [
        ("menu", Scene::Menu),
        ("playing", Scene::Playing),
        ("line-clear", Scene::LineClear),
        ("quad-clear", Scene::QuadClear),
        ("hard-drop", Scene::HardDrop),
        ("stars", Scene::Stars),
        ("plasma", Scene::Plasma),
        ("storm", Scene::Storm),
        ("storm-max", Scene::StormMax),
        ("storm-whiteout", Scene::StormWhiteout),
        ("paused", Scene::Paused),
        ("game-over", Scene::GameOver),
        ("game-over-sweep", Scene::GameOverSweep),
        ("game-over-drop", Scene::GameOverDrop),
        ("options", Scene::Options),
        ("options-paused", Scene::OptionsPaused),
        ("options-audio", Scene::OptionsAudio),
        ("score", Scene::Score),
        ("flat", Scene::Flat),
        ("neon", Scene::Neon),
        ("retro", Scene::Retro),
        ("jelly", Scene::Jelly),
        ("name-entry", Scene::NameEntry),
        ("high-scores", Scene::HighScores),
        ("high-scores-new", Scene::HighScoresNew),
        ("progress-fill", Scene::ProgressFill),
        ("level-up", Scene::LevelUp),
        ("bench", Scene::Bench),
        ("demo", Scene::Demo),
    ];

    fn parse(name: &str) -> Option<Scene> {
        Self::ALL
            .iter()
            .find(|(scene_name, _)| *scene_name == name)
            .map(|(_, scene)| *scene)
    }

    fn names() -> String {
        Self::ALL.map(|(name, _)| name).join(", ")
    }
}

#[derive(Debug)]
pub struct Screenshot {
    pub scene: Scene,
    pub path: PathBuf,
}

impl Screenshot {
    /// parse `--screenshot <scene> [path]`
    pub fn from_args(args: &[String]) -> Result<Option<Screenshot>, String> {
        let Some(i) = args.iter().position(|arg| arg == "--screenshot") else {
            return Ok(None);
        };
        let name = args
            .get(i + 1)
            .ok_or_else(|| format!("--screenshot needs a scene: {}", Scene::names()))?;
        let scene = Scene::parse(name).ok_or_else(|| {
            format!(
                "unknown scene {name:?}, expected one of: {}",
                Scene::names()
            )
        })?;
        let path = args
            .get(i + 2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("screenshots/{name}.png")));
        Ok(Some(Screenshot { scene, path }))
    }
}

fn rtype(c: char) -> Option<RustominoType> {
    match c {
        'I' => Some(RustominoType::I),
        'O' => Some(RustominoType::O),
        'T' => Some(RustominoType::T),
        'L' => Some(RustominoType::L),
        'J' => Some(RustominoType::J),
        'S' => Some(RustominoType::S),
        'Z' => Some(RustominoType::Z),
        _ => None,
    }
}

pub(super) fn fill_rows(board: &mut Board, start: usize, rows: &[&str]) {
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if let Some(rtype) = rtype(c) {
                board.playfield.slots[start + y][x] = SlotState::Locked(rtype);
            }
        }
    }
}

/// a game part way through level 4
pub(super) fn setup_board(world: &mut World) {
    let mut board = world.resource_mut::<Board>();
    fill_rows(&mut board, 0, &STACK);
    board.next = Some(Rustomino::new(RustominoType::L));
    board.held = Some(Rustomino::new(RustominoType::I));
    let mut score = world.resource_mut::<Score>();
    score.level = 4;
    score.lines = 36;
    score.score = 18_200;
    score.score_display = ScoreDisplay::new(score.score);
    score.level_display = ScoreDisplay::new(score.level);
    score.level_progress = LevelProgress::new(LINES_PER_LEVEL, 6);
}

/// stack the board up to near the top
pub(super) fn setup_tall_stack(world: &mut World) {
    let mut board = world.resource_mut::<Board>();
    for start in (STACK.len()..18).step_by(STACK.len()) {
        let rows = &STACK[..STACK.len().min(18 - start)];
        fill_rows(&mut board, start, rows);
    }
}

fn setup_active(world: &mut World) {
    let playfield = &mut world.resource_mut::<Board>().into_inner().playfield;
    playfield.set_active(Rustomino::new(RustominoType::T));
    for _ in 0..8 {
        playfield.translate_active(TranslationDirection::Down);
    }
}

pub(super) fn setup_sample_scores(world: &mut World) {
    let sample = [
        ("ACE", 48_200, 12, 118),
        ("BOB", 31_900, 9, 87),
        ("SCO", 18_200, 4, 36),
        ("ZED", 12_400, 4, 33),
        ("MAX", 6_300, 2, 17),
        ("JOY", 2_100, 1, 8),
    ];
    let mut scores = world.resource_mut::<ScoreBoard>();
    for (initials, score, level, lines) in sample {
        scores.insert(HighScore {
            initials: initials.to_string(),
            score,
            level,
            lines,
        });
    }
}

/// run the effects for the board events so far, then age them
fn run_effects(world: &mut World, time: f32) {
    let _ = world.run_system_cached(on_board_events);
    world.flush();
    let _ = world.run_system_cached_with(step_effects, time);
    world.flush();
}

/// set up a fixed game state for a screenshot, the simulation stays stopped
pub(super) fn setup_scene(world: &mut World, scene: Scene) {
    world.resource_mut::<Time<Virtual>>().pause();
    let mut screen = Screen::Playing;
    world.resource_mut::<Background>().time = 12.0;
    match scene {
        Scene::Menu => screen = Screen::Menu,
        Scene::Options => screen = Screen::Options,
        Scene::OptionsAudio => {
            screen = Screen::Options;
            let mut menus = world.resource_mut::<Menus>();
            menus.audio_open = true;
            menus.audio.next();
        }
        Scene::Score => {
            // part way through the score animation after a clear
            setup_board(world);
            setup_active(world);
            let mut score = world.resource_mut::<Score>();
            score.score += 800;
            score.score_display.add(800);
            score.score_display.update(0.15);
            score.level += 1;
            score.level_display.add(1);
            score.level_display.update(0.15);
        }
        Scene::OptionsPaused => {
            setup_board(world);
            setup_active(world);
            screen = Screen::Options;
            world.resource_mut::<OptionsFrom>().0 = Screen::Paused;
        }
        Scene::Stars | Scene::Plasma | Scene::Storm | Scene::StormMax | Scene::StormWhiteout => {
            let mut background = world.resource_mut::<Background>();
            background.style = match scene {
                Scene::Stars => BackgroundStyle::Stars,
                Scene::Plasma => BackgroundStyle::Plasma,
                _ => BackgroundStyle::Storm,
            };
            if scene == Scene::StormWhiteout {
                background.whiteout = 1.0;
            }
            if matches!(scene, Scene::StormMax | Scene::StormWhiteout) {
                background.storm = 1.0;
                background.sea_storm = 1.0;
                background.wind = 40.0;
                background.rain = 40.0;
                background.sea = 40.0;
            }
            if matches!(scene, Scene::Storm | Scene::StormMax | Scene::StormWhiteout) {
                // catch a lightning strike near its brightest
                // STORM_STRIKE picks the kind of lightning: 0 fork, 1 double, 2 sheet, 3 crawl, 4 distant
                let kind = std::env::var("STORM_STRIKE")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .and_then(|i| crate::draw::StrikeKind::ALL.get(i).copied())
                    .unwrap_or(crate::draw::StrikeKind::Fork);
                background.bolt = crate::draw::storm_strike_time(kind, background.storm);
            }
            setup_board(world);
            setup_active(world);
        }
        Scene::Flat | Scene::Neon | Scene::Retro | Scene::Jelly => {
            let mut style = world.resource_mut::<Style>();
            (style.block, style.board) = match scene {
                Scene::Flat => (BlockStyle::Flat, BoardStyle::Dots),
                Scene::Neon => (BlockStyle::Neon, BoardStyle::Clean),
                Scene::Retro => (BlockStyle::Retro, BoardStyle::Checker),
                _ => (BlockStyle::Jelly, BoardStyle::Grid),
            };
            setup_board(world);
            setup_active(world);
        }
        Scene::Bench => {
            // the benchmark's fps panel over a game
            setup_board(world);
            setup_active(world);
            world
                .resource_mut::<NextState<super::Viewer>>()
                .set(super::Viewer::On);
            let mut debug = world.resource_mut::<super::debug::DebugView>();
            debug.stats_only = true;
            debug.stats.shown = Some(crate::draw::FrameStats {
                draws: 2,
                vertices: 20_000,
                scene_ms: 0.82,
                fps: 1234.0,
                frame_ms: 0.81,
            });
        }
        Scene::Demo => {
            setup_board(world);
            setup_active(world);
            world.resource_mut::<super::attract::Attract>().phase = super::attract::Phase::Demo;
        }
        Scene::Playing | Scene::Paused => {
            setup_board(world);
            setup_active(world);
            if scene == Scene::Paused {
                screen = Screen::Paused;
            }
        }
        Scene::LineClear | Scene::QuadClear => {
            setup_board(world);
            let (lines, origin, progress) = {
                let mut board = world.resource_mut::<Board>();
                if scene == Scene::LineClear {
                    // a vertical I locked into the gap clears the bottom two rows
                    fill_rows(&mut board, 0, &["...I", "...I", "...I", "...I"]);
                    (vec![0, 1], 3, 0.4)
                } else {
                    fill_rows(
                        &mut board,
                        0,
                        &["SSZZOOLJ.T", "OOZZSSLJ.T", "TTJJLLOO.I", "TZZSSJJO.I"],
                    );
                    fill_rows(
                        &mut board,
                        0,
                        &["........I", "........I", "........I", "........I"],
                    );
                    (vec![0, 1, 2, 3], 8, 0.5)
                }
            };
            let count = lines.len();
            world.resource_mut::<Board>().playfield.line_clear = Some(LineClear {
                lines,
                origins: vec![origin],
                time: LINE_CLEAR_DELAY * progress,
                duration: LINE_CLEAR_DELAY,
            });
            world.write_message(BoardEvent::LinesCleared { lines: count });
            run_effects(world, (LINE_CLEAR_DELAY * progress) as f32);
            let level = world.resource::<Score>().level;
            let mut background = world.resource_mut::<Background>();
            background.surge(count);
            background.update(LINE_CLEAR_DELAY * progress, level);
        }
        Scene::HardDrop => {
            // just after a T slams into the stack
            setup_board(world);
            let mut board = world.resource_mut::<Board>();
            board.playfield.set_active(Rustomino::new(RustominoType::T));
            let from = board.playfield.hard_drop_active();
            let to = board.playfield.lock_active();
            if let (Some(from), Some(to)) = (from, to) {
                let color = RustominoType::T.color();
                world.write_message(BoardEvent::HardDropped { from, to, color });
            }
            run_effects(world, 0.04);
        }
        Scene::GameOver | Scene::GameOverSweep | Scene::GameOverDrop | Scene::NameEntry => {
            setup_board(world);
            setup_tall_stack(world);
            let level = world.resource::<Score>().level;
            let time = match scene {
                Scene::GameOverSweep => 0.4,
                Scene::GameOverDrop => 1.3,
                _ => 4.0,
            };
            world.resource_mut::<BoardFx>().game_over();
            run_effects(world, time);
            let mut background = world.resource_mut::<Background>();
            background.freeze(level);
            background.skip(time, level);
            screen = Screen::GameOver;
            if scene == Scene::NameEntry {
                let mut initials = Initials::new("SCO");
                initials.cursor = 1;
                *world.resource_mut::<Initials>() = initials;
                screen = Screen::NameEntry;
            }
        }
        Scene::ProgressFill | Scene::LevelUp => {
            // a double clear popping in, or a quad finishing the bar
            setup_board(world);
            setup_active(world);
            let (lines, time) = if scene == Scene::LevelUp {
                (4, 0.8)
            } else {
                (2, 0.17)
            };
            let level = world.resource::<Score>().level;
            world.resource_mut::<Score>().level_progress.add(lines);
            if scene == Scene::LevelUp {
                world.resource_mut::<Background>().level_up();
            }
            for _ in 0..(time * 60.0) as usize {
                let mut score = world.resource_mut::<Score>();
                if score.level_progress.update(1.0 / 60.0) {
                    score.level_display.add(1);
                }
                world.resource_mut::<Background>().update(1.0 / 60.0, level);
            }
        }
        Scene::HighScores | Scene::HighScoresNew => {
            setup_sample_scores(world);
            if scene == Scene::HighScoresNew {
                world.resource_mut::<ScoreView>().highlight = Some(2);
            }
            screen = Screen::HighScores;
        }
    }
    world
        .resource_mut::<NextState<Screen>>()
        .set_if_different(screen);
}

/// rgba8 pixels to a png
pub fn save_png(pixels: &[u8], width: u32, height: u32, path: &Path) -> GameResult {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let png_error = |e: png::EncodingError| GameError(e.to_string());
    let mut encoder = png::Encoder::new(BufWriter::new(fs::File::create(path)?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(pixels).map_err(png_error)?;
    writer.finish().map_err(png_error)
}
