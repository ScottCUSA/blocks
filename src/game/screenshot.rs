use std::{fs, io::BufWriter, path::Path, path::PathBuf};
use strum::{EnumString, VariantArray, VariantNames};

use ggez::{
    graphics::{Canvas, Image, ImageFormat},
    Context, GameError, GameResult,
};

use super::{BlocksState, GameState, LINES_PER_LEVEL, LINE_CLEAR_DELAY};
use crate::background::BackgroundStyle;
use crate::draw::{storm_strike_time, StrikeKind};
use crate::level_progress::LevelProgress;
use crate::score_display::ScoreDisplay;
use crate::scores::{HighScore, Initials};
use crate::style::{BlockStyle, BoardStyle};
use crate::{
    playfield::{self, SlotState, TranslationDirection},
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

#[derive(Debug, Clone, Copy, PartialEq, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case")]
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
    Demo,
}

fn scene_names() -> String {
    Scene::VARIANTS.join(", ")
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
            .ok_or_else(|| format!("--screenshot needs a scene: {}", scene_names()))?;
        let scene = name
            .parse()
            .map_err(|_| format!("unknown scene {name:?}, expected one of: {}", scene_names()))?;
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

impl BlocksState {
    pub(super) fn fill_rows(&mut self, start: usize, rows: &[&str]) {
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if let Ok(rtype) = c.to_string().parse() {
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
        use Scene::*;
        self.background.time = 12.0;
        // most scenes are a game in progress, with a piece falling
        let menus = matches!(
            scene,
            Menu | Options | OptionsAudio | HighScores | HighScoresNew
        );
        let landed = matches!(
            scene,
            LineClear | QuadClear | HardDrop | GameOver | GameOverSweep | GameOverDrop | NameEntry
        );
        if !menus {
            self.setup_board();
            self.state = GameState::Playing;
        }
        if !menus && !landed {
            self.setup_active();
        }
        match scene {
            Menu => self.state = GameState::Menu,
            Options => self.state = GameState::Options,
            OptionsAudio => {
                self.state = GameState::Options;
                self.audio_open = true;
                self.audio_state.next();
            }
            OptionsPaused => {
                self.state = GameState::Options;
                self.previous_state = GameState::Paused;
                return;
            }
            Playing => {}
            Paused => self.state = GameState::Paused,
            Demo => self.attract.phase = super::attract::Phase::Demo,
            Score => {
                // part way through the score animation after a clear
                self.score += 800;
                self.score_display.add(800);
                self.score_display.update(0.15);
                self.level += 1;
                self.level_display.add(1);
                self.level_display.update(0.15);
            }
            Stars | Plasma | Storm | StormMax | StormWhiteout => {
                self.background.style = match scene {
                    Stars => BackgroundStyle::Stars,
                    Plasma => BackgroundStyle::Plasma,
                    _ => BackgroundStyle::Storm,
                };
                if scene == StormWhiteout {
                    self.background.whiteout = 1.0;
                }
                if matches!(scene, StormMax | StormWhiteout) {
                    self.background.storm = 1.0;
                    self.background.sea_storm = 1.0;
                    self.background.wind = 40.0;
                    self.background.rain = 40.0;
                    self.background.sea = 40.0;
                }
                if matches!(scene, Storm | StormMax | StormWhiteout) {
                    // catch a lightning strike near its brightest
                    // STORM_STRIKE picks the kind of lightning: 0 fork, 1 double, 2 sheet, 3 crawl, 4 distant
                    let kind = std::env::var("STORM_STRIKE")
                        .ok()
                        .and_then(|v| v.parse::<usize>().ok())
                        .and_then(|i| StrikeKind::VARIANTS.get(i).copied())
                        .unwrap_or(StrikeKind::Fork);
                    self.background.bolt = storm_strike_time(kind, self.background.storm);
                }
            }
            Flat | Neon | Retro | Jelly => {
                (self.style.block, self.style.board) = match scene {
                    Flat => (BlockStyle::Flat, BoardStyle::Dots),
                    Neon => (BlockStyle::Neon, BoardStyle::Clean),
                    Retro => (BlockStyle::Retro, BoardStyle::Checker),
                    _ => (BlockStyle::Jelly, BoardStyle::Grid),
                };
            }
            LineClear | QuadClear => {
                // a vertical I locked into the gap clears the bottom rows
                let (lines, origin, at) = if scene == QuadClear {
                    self.fill_rows(0, &["SSZZOOLJ.T", "OOZZSSLJ.T", "TTJJLLOO.I", "TZZSSJJO.I"]);
                    self.fill_rows(0, &["........I"; 4]);
                    (4, 8, 0.5)
                } else {
                    self.fill_rows(0, &["...I"; 4]);
                    (2, 3, 0.4)
                };
                let time = LINE_CLEAR_DELAY * at;
                self.playfield.line_clear = Some(playfield::LineClear {
                    lines: (0..lines).collect(),
                    origins: vec![origin],
                    time,
                    duration: LINE_CLEAR_DELAY,
                });
                self.effects.line_clear(&self.playfield);
                self.effects.update(time);
                self.background.surge(lines);
                self.background.update(time, self.level);
            }
            HardDrop => {
                // just after a T slams into the stack
                self.playfield.set_active(Rustomino::new(RustominoType::T));
                if let Some(from) = self.playfield.hard_drop_active() {
                    if let Some(to) = self.playfield.lock_active() {
                        self.effects.hard_drop(&from, &to, RustominoType::T.color());
                    }
                }
                self.effects.update(0.04);
            }
            GameOver | GameOverSweep | GameOverDrop | NameEntry => {
                self.setup_tall_stack();
                self.effects.game_over();
                self.background.freeze(self.level);
                let time = match scene {
                    GameOverSweep => 0.4,
                    GameOverDrop => 1.3,
                    _ => 4.0,
                };
                self.effects.update(time as f64);
                self.background.skip(time, self.level);
                self.state = GameState::GameOver;
                if scene == NameEntry {
                    self.initials = Initials::new("SCO");
                    self.initials.cursor = 1;
                    self.state = GameState::NameEntry;
                }
            }
            ProgressFill | LevelUp => {
                // a double clear popping in, or a quad finishing the bar
                let (lines, time) = if scene == LevelUp {
                    (4, 0.8)
                } else {
                    (2, 0.17)
                };
                self.level_progress.add(lines);
                if scene == LevelUp {
                    self.background.level_up();
                }
                for _ in 0..(time * 60.0) as usize {
                    if self.level_progress.update(1.0 / 60.0) {
                        self.level_display.add(1);
                    }
                    self.background.update(1.0 / 60.0, self.level);
                }
            }
            HighScores | HighScoresNew => {
                self.setup_sample_scores();
                if scene == HighScoresNew {
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
