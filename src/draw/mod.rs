use crate::gfx::{Color, Mesh, Rect, Vec2, Vertex};
use crate::GameResult;

use crate::playfield;
use crate::style::BlockStyle;

mod background;
mod board;
mod paint;
mod painter;
mod screens;
mod sprites;
mod text;
mod widgets;

use paint::*;
use painter::*;
use text::*;
use widgets::*;

pub use background::{clear_color, draw_background, storm_strike_time, StrikeKind};
pub use board::GameLayer;
pub use painter::Painter;
pub use screens::{
    DebugScreen, FrameStats, GameOverScreen, HighScoresScreen, MenuScreen, NameEntryScreen,
    OptionRow, OptionsScreen, PausedScreen,
};
pub use sprites::{BoardShown, BoardSprites, SpritesPlugin};
pub use text::set_pixel_scale;
pub use widgets::Overlay;

const BLOCK_SIZE: f32 = 30.;
const BLOCK_PADDING: f32 = 1.;
const CELL: f32 = BLOCK_SIZE + BLOCK_PADDING;
const STAGING_PADDING: f32 = 2.;
pub const VIEW_WIDTH: f32 = 1024.0;
pub const VIEW_HEIGHT: f32 = 768.;
const UI_FONT_SIZE: f32 = 24.0;
pub const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const FRAME_COLOR: Color = Color::new(0.55, 0.8, 0.9, 0.35);
const FRAME_WIDTH: f32 = 3.0;
const JELLY_RADIUS: f32 = 8.0; // at BLOCK_SIZE
const GHOST_OUTLINE_WIDTH: f32 = 2.0;
const LABEL_COLOR: Color = Color::new(0.7, 0.88, 1.0, 1.0);
const TEXT_SHADOW_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HIGHLIGHT_COLOR: Color = Color::new(1.0, 0.92, 0.72, 0.15);
const ACCENT_COLOR: Color = LINE_CLEAR_COLOR;
const SLIDER_EMPTY_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.12);
const TITLE_TINT: f32 = 0.05; // how far title letters are lightened from the piece colors
const MENU_ITEM_SIZE: f32 = 40.0;
const MENU_ROW_HEIGHT: f32 = 60.0;
const PANEL_PADDING: f32 = 20.0;
const SMALL_TEXT_SIZE: f32 = 16.0;
const PREVIEW_PADDING: f32 = 12.0;
const SLIDER_SEGMENT: f32 = 18.0;
const SLIDER_GAP: f32 = 4.0;
const PROGRESS_HEIGHT: f32 = 7.0; // level progress segment height
const PROGRESS_GAP: f32 = 6.0; // between the playfield frame and progress frame
const LINE_CLEAR_COLOR: Color = Color::new(1.0, 0.92, 0.72, 1.0); // soft gold

/// what every layer draws with
pub struct Frame<'a> {
    pub time: f32, // seconds since start
    pub meshes: &'a Meshes,
    pub view: &'a ViewSettings,
    pub block: BlockStyle,
}

/// a screen or overlay, drawn over the background
pub trait Layer {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult;
}

#[derive(Debug, bevy::prelude::Resource)]
pub struct ViewSettings {
    pub scale: f32, // physical pixels per view unit
    pub view_rect: Rect,
    pub playfield_rect: Rect,
    pub staging_rect: Rect,
    pub preview_rect: Rect,
    pub hold_rect: Rect,
    pub score_label_pos: Vec2,
    pub level_label_pos: Vec2,
    pub title_pos: Vec2,
    pub level_pos: Vec2,
    pub progress_rect: Rect,
    pub score_pos: Vec2,
}

impl ViewSettings {
    /// layout for a drawable in physical pixels, scaled to fit the window
    pub fn new(drawable_width: f32, drawable_height: f32) -> Self {
        let fit = (drawable_width / VIEW_WIDTH).min(drawable_height / VIEW_HEIGHT);
        // whole pixels per cell, so every gap is the same width
        let scale = (fit * CELL).floor().max(1.0) / CELL;
        let snap = |v: f32| (v * scale).round() / scale;
        let drawable_width = drawable_width / scale;
        let drawable_height = drawable_height / scale;

        // calculate the playfield dimensions based on block size, padding and playfield slots
        let playfield_w =
            (playfield::PLAYFIELD_SLOTS[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let playfield_h = ((playfield::PLAYFIELD_SLOTS[1] - 2) as f32
            * (BLOCK_SIZE + BLOCK_PADDING))
            + BLOCK_PADDING;

        // calculate the dimentions of the staging area
        let staging_w = playfield_w;
        let staging_h = (2. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        // calculate the dimensions of the preview area
        let preview_w = (4. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING + 2.0 * PREVIEW_PADDING;
        let preview_h = staging_h + 2.0 * PREVIEW_PADDING;
        // calculate the dimensions of the hold area
        let hold_w = preview_w;
        let hold_h = preview_h;

        // center playfield
        let playfield_x = snap(drawable_width / 2.0 - playfield_w / 2.0);
        let playfield_y = snap(drawable_height / 2.0 - playfield_h / 2.0 + staging_h / 2.0 + 1.0);
        // center staging area above playfield
        let staging_x = playfield_x;
        let staging_y = playfield_y - staging_h - STAGING_PADDING;
        // center preview area to the right of playfield
        let preview_x = snap(playfield_x + playfield_w + 10.0);
        let preview_y = playfield_y;
        // center hold area to the left of playfield
        let hold_x = snap(playfield_x - preview_w - 10.0);
        let hold_y = playfield_y;

        Self {
            scale,
            view_rect: Rect::new(0., 0., drawable_width, drawable_height),
            playfield_rect: Rect::new(playfield_x, playfield_y, playfield_w, playfield_h),
            staging_rect: Rect::new(staging_x, staging_y, staging_w, staging_h),
            preview_rect: Rect::new(preview_x, preview_y, preview_w, preview_h),
            hold_rect: Rect::new(hold_x, hold_y, hold_w, hold_h),
            score_label_pos: Vec2::new(
                playfield_x + playfield_w + 30.0,
                playfield_y + playfield_h - 30.0,
            ),
            level_label_pos: Vec2::new(playfield_x - 180.0, playfield_y + playfield_h - 30.0),
            title_pos: Vec2::new(hold_x, staging_y - 4.0),
            level_pos: Vec2::new(playfield_x - 60.0, playfield_y + playfield_h - 30.0),
            progress_rect: Rect::new(
                playfield_x,
                playfield_y + playfield_h + 2.0 * FRAME_WIDTH + PROGRESS_GAP,
                playfield_w,
                PROGRESS_HEIGHT + 2.0 * BLOCK_PADDING,
            ),
            score_pos: Vec2::new(
                playfield_x + playfield_w + 150.0,
                playfield_y + playfield_h - 30.0,
            ),
        }
    }
}

/// meshes reused every frame
#[derive(bevy::prelude::Resource)]
pub struct Meshes {
    rect: Mesh,       // unit rectangle, scaled and colored per draw
    background: Mesh, // unit rectangle fading out downward, tinted per draw
    rounded: Mesh,    // BLOCK_SIZE rounded square, scaled per draw
    glow: Mesh,       // unit disc fading to clear at the rim
}

impl Meshes {
    pub fn new() -> Self {
        let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
        let block = Rect::new(0.0, 0.0, BLOCK_SIZE, BLOCK_SIZE);
        let mut background = Mesh::rect(unit, Color::WHITE);
        for v in &mut background.vertices[2..] {
            v.color = Color::new(1.0, 1.0, 1.0, 0.0);
        }
        Self {
            rect: Mesh::rect(unit, Color::WHITE),
            rounded: Mesh::rounded_rect(block, JELLY_RADIUS, Color::WHITE),
            background,
            glow: glow_mesh(),
        }
    }
}

/// soft disc, rings of falling alpha
fn glow_mesh() -> Mesh {
    const RINGS: [(f32, f32); 4] = [(0.0, 1.0), (0.35, 0.6), (0.7, 0.2), (1.0, 0.0)];
    const SEGMENTS: u32 = 48;
    let mut vertices = Vec::new();
    for (radius, alpha) in RINGS {
        let count = if radius == 0.0 { 1 } else { SEGMENTS };
        for k in 0..count {
            let a = k as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            vertices.push(Vertex {
                position: Vec2::new(a.cos() * radius, a.sin() * radius),
                color: Color::new(1.0, 1.0, 1.0, alpha),
            });
        }
    }
    let ring = |r: u32, k: u32| 1 + (r - 1) * SEGMENTS + k % SEGMENTS;
    let mut indices = Vec::new();
    for k in 0..SEGMENTS {
        indices.extend([0, ring(1, k), ring(1, k + 1)]);
        for r in 1..RINGS.len() as u32 - 1 {
            let (a, b) = (ring(r, k), ring(r, k + 1));
            let (c, d) = (ring(r + 1, k), ring(r + 1, k + 1));
            indices.extend([a, c, d, a, d, b]);
        }
    }
    Mesh { vertices, indices }
}
