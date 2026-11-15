use std::{fs, io::BufWriter, path::Path, path::PathBuf};

use ggez::{
    graphics::{Canvas, Image, ImageFormat},
    Context, GameError, GameResult,
};

use super::{BlocksState, GameState, LINES_PER_LEVEL, LINE_CLEAR_DELAY};
use crate::background::BackgroundStyle;
use crate::level_progress::LevelProgress;
use crate::score_display::ScoreDisplay;
use crate::scores::{HighScore, Initials};
use crate::style::{BlockStyle, BoardStyle};
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
    Paused,
    GameOver,
    GameOverSweep,
    GameOverDrop,
    Options,
    OptionsPaused,
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
}

impl Scene {
    const ALL: [(&'static str, Scene); 23] = [
        ("menu", Scene::Menu),
        ("playing", Scene::Playing),
        ("line-clear", Scene::LineClear),
        ("quad-clear", Scene::QuadClear),
        ("hard-drop", Scene::HardDrop),
        ("stars", Scene::Stars),
        ("plasma", Scene::Plasma),
        ("paused", Scene::Paused),
        ("game-over", Scene::GameOver),
        ("game-over-sweep", Scene::GameOverSweep),
        ("game-over-drop", Scene::GameOverDrop),
        ("options", Scene::Options),
        ("options-paused", Scene::OptionsPaused),
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
    pub(super) scene: Scene,
    path: PathBuf,
    image: Option<Image>, // rendered, saved on the next frame
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
        Ok(Some(Screenshot {
            scene,
            path,
            image: None,
        }))
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

impl BlocksState {
    pub(super) fn fill_rows(&mut self, start: usize, rows: &[&str]) {
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if let Some(rtype) = rtype(c) {
                    self.playfield.slots[start + y][x] = SlotState::Locked(rtype);
                }
            }
        }
    }

    pub(super) fn setup_board(&mut self) {
        self.fill_rows(0, &STACK);
        self.next_rustomino = Some(Rustomino::new(RustominoType::L));
        self.held_rustomino = Some(Rustomino::new(RustominoType::I));
        self.level = 4;
        self.total_lines_cleared = 36;
        self.score = 18_200;
        self.score_display = ScoreDisplay::new(self.score);
        self.level_display = ScoreDisplay::new(self.level);
        self.level_progress = LevelProgress::new(LINES_PER_LEVEL, 6);
    }

    /// stack the board up to near the top
    pub(super) fn setup_tall_stack(&mut self) {
        for start in (STACK.len()..18).step_by(STACK.len()) {
            let rows = &STACK[..STACK.len().min(18 - start)];
            self.fill_rows(start, rows);
        }
    }

    fn setup_active(&mut self) {
        self.playfield.set_active(Rustomino::new(RustominoType::T));
        for _ in 0..8 {
            self.playfield.translate_active(TranslationDirection::Down);
        }
        self.playfield.translate_active(TranslationDirection::Right);
    }

    pub(super) fn setup_sample_scores(&mut self) {
        let sample = [
            ("ACE", 48_200, 12, 118),
            ("BOB", 31_900, 9, 87),
            ("SCO", 18_200, 4, 36),
            ("ZED", 12_400, 4, 33),
            ("MAX", 6_300, 2, 17),
            ("JOY", 2_100, 1, 8),
        ];
        for (initials, score, level, lines) in sample {
            self.scores.insert(HighScore {
                initials: initials.to_string(),
                score,
                level,
                lines,
            });
        }
    }

    /// set up a fixed game state for a screenshot
    pub(super) fn setup_scene(&mut self, scene: Scene) {
        self.background.time = 12.0;
        match scene {
            Scene::Menu => self.state = GameState::Menu,
            Scene::Options => self.state = GameState::Options,
            Scene::Score => {
                // part way through the score animation after a clear
                self.setup_board();
                self.setup_active();
                self.score += 800;
                self.score_display.add(800);
                self.score_display.update(0.15);
                self.level += 1;
                self.level_display.add(1);
                self.level_display.update(0.15);
                self.state = GameState::Playing;
            }
            Scene::OptionsPaused => {
                self.setup_board();
                self.setup_active();
                self.state = GameState::Options;
                self.previous_state = GameState::Paused;
                return;
            }
            Scene::Stars | Scene::Plasma => {
                self.background.style = if scene == Scene::Stars {
                    BackgroundStyle::Stars
                } else {
                    BackgroundStyle::Plasma
                };
                self.setup_board();
                self.setup_active();
                self.state = GameState::Playing;
            }
            Scene::Flat | Scene::Neon | Scene::Retro | Scene::Jelly => {
                (self.style.block, self.style.board) = match scene {
                    Scene::Flat => (BlockStyle::Flat, BoardStyle::Dots),
                    Scene::Neon => (BlockStyle::Neon, BoardStyle::Clean),
                    Scene::Retro => (BlockStyle::Retro, BoardStyle::Checker),
                    _ => (BlockStyle::Jelly, BoardStyle::Grid),
                };
                self.setup_board();
                self.setup_active();
                self.state = GameState::Playing;
            }
            Scene::Playing | Scene::Paused => {
                self.setup_board();
                self.setup_active();
                self.state = if scene == Scene::Paused {
                    GameState::Paused
                } else {
                    GameState::Playing
                };
            }
            Scene::LineClear => {
                self.setup_board();
                // a vertical I locked into the gap clears the bottom two rows
                self.fill_rows(0, &["...I", "...I", "...I", "...I"]);
                self.playfield.line_clear = Some(LineClear {
                    lines: vec![0, 1],
                    origins: vec![3],
                    time: LINE_CLEAR_DELAY * 0.4,
                    duration: LINE_CLEAR_DELAY,
                });
                self.effects.line_clear(&self.playfield);
                self.effects.update(LINE_CLEAR_DELAY * 0.4);
                self.background.surge(2);
                self.background.update(LINE_CLEAR_DELAY * 0.4, self.level);
                self.state = GameState::Playing;
            }
            Scene::QuadClear => {
                self.setup_board();
                self.fill_rows(0, &["SSZZOOLJ.T", "OOZZSSLJ.T", "TTJJLLOO.I", "TZZSSJJO.I"]);
                self.fill_rows(0, &["........I", "........I", "........I", "........I"]);
                self.playfield.line_clear = Some(LineClear {
                    lines: vec![0, 1, 2, 3],
                    origins: vec![8],
                    time: LINE_CLEAR_DELAY * 0.5,
                    duration: LINE_CLEAR_DELAY,
                });
                self.effects.line_clear(&self.playfield);
                self.effects.update(LINE_CLEAR_DELAY * 0.5);
                self.background.surge(4);
                self.background.update(LINE_CLEAR_DELAY * 0.5, self.level);
                self.state = GameState::Playing;
            }
            Scene::HardDrop => {
                // just after a T slams into the stack
                self.setup_board();
                self.playfield.set_active(Rustomino::new(RustominoType::T));
                if let Some(from) = self.playfield.hard_drop_active() {
                    if let Some(to) = self.playfield.lock_active() {
                        self.effects.hard_drop(&from, &to, RustominoType::T.color());
                    }
                }
                self.effects.update(0.04);
                self.state = GameState::Playing;
            }
            Scene::GameOver | Scene::GameOverSweep | Scene::GameOverDrop | Scene::NameEntry => {
                self.setup_board();
                self.setup_tall_stack();
                self.effects.game_over();
                self.background.freeze(self.level);
                let time = match scene {
                    Scene::GameOverSweep => 0.4,
                    Scene::GameOverDrop => 1.3,
                    _ => 4.0,
                };
                self.effects.update(time as f64);
                self.background.skip(time, self.level);
                self.state = GameState::GameOver;
                if scene == Scene::NameEntry {
                    self.initials = Initials::new("SCO");
                    self.initials.cursor = 1;
                    self.state = GameState::NameEntry;
                }
            }
            Scene::ProgressFill | Scene::LevelUp => {
                // a double clear popping in, or a quad finishing the bar
                self.setup_board();
                self.setup_active();
                let (lines, time) = if scene == Scene::LevelUp {
                    (4, 0.8)
                } else {
                    (2, 0.17)
                };
                self.level_progress.add(lines);
                if scene == Scene::LevelUp {
                    self.background.level_up();
                }
                for _ in 0..(time * 60.0) as usize {
                    if self.level_progress.update(1.0 / 60.0) {
                        self.level_display.add(1);
                    }
                    self.background.update(1.0 / 60.0, self.level);
                }
                self.state = GameState::Playing;
            }
            Scene::HighScores | Scene::HighScoresNew => {
                self.setup_sample_scores();
                if scene == Scene::HighScoresNew {
                    self.highlight = Some(2);
                }
                self.state = GameState::HighScores;
            }
        }
        self.previous_state = self.state;
    }

    /// render the screenshot, then save it on the following frame
    /// returns true once it has been saved
    pub(super) fn capture_screenshot(&mut self, ctx: &mut Context) -> GameResult<bool> {
        let Some(screenshot) = &mut self.screenshot else {
            return Ok(false);
        };
        // pixels can only be read after the frame's gpu work is submitted
        if let Some(image) = screenshot.image.take() {
            save_png(ctx, &image, &screenshot.path)?;
            println!("saved screenshot to {}", screenshot.path.display());
            return Ok(true);
        }

        let (width, height) = ctx.gfx.drawable_size();
        let image = Image::new_canvas_image(ctx, width as u32, height as u32, 1);
        let mut canvas = Canvas::from_image(
            ctx,
            image.clone(),
            crate::draw::clear_color(self.background.style),
        );
        self.draw_scene(ctx, &mut canvas)?;
        canvas.finish(ctx)?;
        if let Some(screenshot) = &mut self.screenshot {
            screenshot.image = Some(image);
        }
        Ok(false)
    }
}

fn save_png(ctx: &Context, image: &Image, path: &Path) -> GameResult {
    let mut pixels = image.to_pixels(ctx)?;
    match image.format() {
        ImageFormat::Rgba8Unorm | ImageFormat::Rgba8UnormSrgb => {}
        ImageFormat::Bgra8Unorm | ImageFormat::Bgra8UnormSrgb => {
            for pixel in pixels.as_chunks_mut::<4>().0 {
                pixel.swap(0, 2);
            }
        }
        format => {
            return Err(GameError::RenderError(format!(
                "cannot save a screenshot in the {format:?} format"
            )))
        }
    }

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let png_error = |e: png::EncodingError| GameError::CustomError(e.to_string());
    let mut encoder = png::Encoder::new(
        BufWriter::new(fs::File::create(path)?),
        image.width(),
        image.height(),
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(&pixels).map_err(png_error)?;
    writer.finish().map_err(png_error)
}
