use std::{fs, io::BufWriter, path::Path, path::PathBuf};

use ggez::{
    graphics::{Canvas, Image, ImageFormat},
    Context, GameError, GameResult,
};

use super::{BlocksState, GameState, LINE_CLEAR_DELAY};
use crate::score_display::ScoreDisplay;
use crate::{
    draw::BACKGROUND_COLOR,
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
    Paused,
    GameOver,
    Options,
    OptionsPaused,
    Score,
}

impl Scene {
    const ALL: [(&'static str, Scene); 8] = [
        ("menu", Scene::Menu),
        ("playing", Scene::Playing),
        ("line-clear", Scene::LineClear),
        ("paused", Scene::Paused),
        ("game-over", Scene::GameOver),
        ("options", Scene::Options),
        ("options-paused", Scene::OptionsPaused),
        ("score", Scene::Score),
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
    fn fill_rows(&mut self, start: usize, rows: &[&str]) {
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if let Some(rtype) = rtype(c) {
                    self.playfield.slots[start + y][x] = SlotState::Locked(rtype);
                }
            }
        }
    }

    fn setup_board(&mut self) {
        self.fill_rows(0, &STACK);
        self.next_rustomino = Some(Rustomino::new(RustominoType::L));
        self.held_rustomino = Some(Rustomino::new(RustominoType::I));
        self.level = 4;
        self.score = 18_200;
        self.score_display = ScoreDisplay::new(self.score);
    }

    fn setup_active(&mut self) {
        self.playfield.set_active(Rustomino::new(RustominoType::T));
        for _ in 0..8 {
            self.playfield.translate_active(TranslationDirection::Down);
        }
        self.playfield.translate_active(TranslationDirection::Right);
    }

    /// set up a fixed game state for a screenshot
    pub(super) fn setup_scene(&mut self, scene: Scene) {
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
                self.state = GameState::Playing;
            }
            Scene::OptionsPaused => {
                self.setup_board();
                self.setup_active();
                self.state = GameState::Options;
                self.previous_state = GameState::Paused;
                return;
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
                self.state = GameState::Playing;
            }
            Scene::GameOver => {
                self.setup_board();
                for start in (STACK.len()..18).step_by(STACK.len()) {
                    let rows = &STACK[..STACK.len().min(18 - start)];
                    self.fill_rows(start, rows);
                }
                self.state = GameState::GameOver;
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
        let mut canvas = Canvas::from_image(ctx, image.clone(), BACKGROUND_COLOR);
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
            for pixel in pixels.chunks_exact_mut(4) {
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
