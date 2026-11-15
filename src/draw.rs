use ggez::glam::{IVec2, Vec2};
use ggez::graphics::{self, Canvas, Color, DrawMode, Rect, TextFragment};
use ggez::{Context, GameResult};

use crate::background::{Background, BackgroundStyle, STAR_FIELDS};
use crate::effects::{Effects, GameOver};
use crate::game::Mode;
use crate::level_progress::LevelProgress;
use crate::menus::{self, Menu};
use crate::playfield::{self, LineClear, Playfield, SlotState};
use crate::rustomino::{Rustomino, RustominoType};
use crate::score_display::ScoreDisplay;
use crate::scores::{Initials, ScoreBoard, MAX_INITIALS, MAX_SCORES};
use crate::style::{BlockStyle, BoardStyle, Style};
use crate::util;
use std::sync::atomic::{AtomicU32, Ordering};
use strum::IntoEnumIterator;

const BLOCK_SIZE: f32 = 30.;
const BLOCK_PADDING: f32 = 1.;
const CELL: f32 = BLOCK_SIZE + BLOCK_PADDING;
const STAGING_PADDING: f32 = 2.;

pub const VIEW_WIDTH: f32 = 1024.0;
pub const VIEW_HEIGHT: f32 = 768.;
const UI_FONT_SIZE: f32 = 24.0;
pub const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
// top and bottom of the window per background
const STARS_COLORS: (Color, Color) = (
    Color::new(0.01, 0.01, 0.04, 1.0),
    Color::new(0.06, 0.04, 0.16, 1.0),
);
const PLASMA_COLORS: (Color, Color) = (
    Color::new(0.02, 0.01, 0.06, 1.0),
    Color::new(0.1, 0.04, 0.19, 1.0),
);
const OFF_COLORS: (Color, Color) = (
    Color::new(0.06, 0.07, 0.09, 1.0),
    Color::new(0.16, 0.18, 0.22, 1.0),
);
const FRAME_COLOR: Color = Color::new(0.55, 0.8, 0.9, 0.35);
const FRAME_WIDTH: f32 = 3.0;
const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.06);
const DOT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.18);
const DOT_SIZE: f32 = 3.0;
const CHECKER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.035);
const NEON_EDGE: f32 = 3.0;
const RETRO_EDGE: f32 = 2.0;
const JELLY_RADIUS: f32 = 8.0; // at BLOCK_SIZE
const BEVEL: f32 = 3.0; // block edge highlight and shadow width
const GHOST_FILL_ALPHA: f32 = 0.12;
const GHOST_OUTLINE_ALPHA: f32 = 0.6;
const GHOST_OUTLINE_WIDTH: f32 = 2.0;
const LABEL_COLOR: Color = Color::new(0.7, 0.88, 1.0, 1.0);
const TEXT_SHADOW_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PANEL_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
const HIGHLIGHT_COLOR: Color = Color::new(1.0, 0.92, 0.72, 0.15);
const ACCENT_COLOR: Color = LINE_CLEAR_COLOR;
const SLIDER_EMPTY_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.12);
const TITLE_SIZE: f32 = 96.0;
const TITLE_TINT: f32 = 0.05; // how far title letters are lightened from the piece colors
const MENU_ITEM_SIZE: f32 = 40.0;
const OPTION_LABEL_SIZE: f32 = 32.0;
const MENU_ROW_HEIGHT: f32 = 60.0;
const MENU_PANEL_WIDTH: f32 = 520.0;
const OPTIONS_PANEL_WIDTH: f32 = 680.0;
const OPTION_ROW_HEIGHT: f32 = 40.0;
const PANEL_PADDING: f32 = 20.0;
const SMALL_TEXT_SIZE: f32 = 16.0;
const PAUSED_TITLE_SIZE: f32 = 64.0;
const PREVIEW_PADDING: f32 = 12.0;
const CONTROLS_ROW_HEIGHT: f32 = 22.0;
const SLIDER_SEGMENT: f32 = 18.0;
const SLIDER_GAP: f32 = 4.0;
const PROGRESS_HEIGHT: f32 = 7.0; // level progress segment height
const PROGRESS_GAP: f32 = 6.0; // between the playfield frame and progress frame
const SCORE_POP_SCALE: f32 = 0.3; // how much the score grows when it increases
const SCORE_POPUP_OFFSET: f32 = 34.0; // where "+points" starts above the score
const SCORE_POPUP_RISE: f32 = 40.0; // how far "+points" floats up
const CONTROLS: [(&str, &str); 6] = [
    ("Move", "Left/Right  A/D"),
    ("Rotate", "Up/W  Ctrl/Z"),
    ("Soft drop", "Down/S"),
    ("Hard drop", "Space"),
    ("Hold", "Shift/C"),
    ("Pause", "Esc"),
];
const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
const QUAD_GLINT_LEAD: f32 = 0.06; // glint peak after a block starts fading, fraction of the clear
const QUAD_GLINT_WIDTH: f32 = 0.12; // fraction of the clear
const QUAD_GLINT_ROW: f32 = 0.025; // delay per row up
const CALLOUT_SIZE: f32 = 32.0;
const CALLOUT_QUAD_SIZE: f32 = 48.0;
const CALLOUT_POP_SCALE: f32 = 0.5;
const TRAIL_SEGMENTS: usize = 8;
const STAR_COUNT: usize = 400;
const STAR_FLIGHT: f32 = 0.06; // depth per second
const STAR_NEAR: f32 = 0.02;
const STAR_SPREAD: f32 = 0.5; // of the view at full depth
const NEBULA_CLOUDS: usize = 4;
const NEBULA_BLOBS: usize = 7; // per cloud
const NEBULA_ALPHA: f32 = 0.09;
const NEBULA_DRIFT: f32 = 0.05;
const NEBULA_COLORS: [Color; 5] = [
    Color::new(0.55, 0.25, 0.85, 1.0), // violet
    Color::new(0.85, 0.3, 0.6, 1.0),   // rose
    Color::new(0.25, 0.4, 0.95, 1.0),  // indigo
    Color::new(0.2, 0.75, 0.8, 1.0),   // teal
    Color::new(0.9, 0.55, 0.35, 1.0),  // amber
];
const STARS_STILL_TWINKLE: f32 = 0.5; // twinkle depth once stopped
const STARS_SPARKLE: f32 = 0.3; // extra brightness at the peak
const STARS_SPARKLE_CHANCE: f32 = 0.05;
const STARS_SPARKLE_PERIOD: f32 = 3.0; // seconds
const STARS_SPARKLE_TIME: f32 = 0.5;
const FROST_COLOR: Color = Color::new(0.7, 0.85, 1.0, 1.0);
const MELT_ORDER: f32 = 0.6; // of the freeze timing
const MELT_FADE: f32 = 0.4;
const CRYSTAL_SEEDS: usize = 6;
const CRYSTAL_GROW: f32 = 0.06; // seconds per cell
const CRYSTAL_JITTER: f32 = 0.25;
const CRYSTAL_FLASH: f32 = 0.35;
const GLINT_PERIOD: f32 = 5.0;
const GLINT_CHANCE: f32 = 0.04;
const GLINT_TIME: f32 = 0.5;
const STAR_COLOR: Color = Color::new(0.8, 0.9, 1.0, 1.0);
const PLASMA_CELL: f32 = 32.0; // px
const RING_WIDTH: f32 = 70.0; // px, level up ring
const WARP_STREAK: f32 = 0.4; // flight per px from center
const WARP_ARRIVE: f32 = 0.35; // new stars start appearing
const WARP_DOTS: usize = 48; // max per streak
const PLASMA_SPEED: f32 = 0.8;
const TRAIL_ALPHA: f32 = 0.45;
const IMPACT_ALPHA: f32 = 0.6;
const LINE_CLEAR_COLOR: Color = Color::new(1.0, 0.92, 0.72, 1.0); // soft gold
const CURTAIN_FLASH: f32 = 0.6;
const PREVIEW_ROW: i32 = playfield::PLAYFIELD_SLOTS[1] as i32; // curtain reaches hold and next last
const GAME_OVER_TITLE_SIZE: f32 = 56.0;
const GAME_OVER_TITLE_ROW: f32 = 100.0;
const GAME_OVER_STAT_ROW: f32 = 44.0;
const GAME_OVER_STAT_SIZE: f32 = 28.0;
const LETTER_DROP: f32 = 140.0; // px
const LETTER_WAVE: f32 = 4.0; // px
const SPARKLE_COUNT: usize = 14;
const SPARKLE_SIZE: f32 = 8.0;
const BEST_OFFSET: f32 = 100.0; // above the score, clear of the popups
const NAME_TITLE_SIZE: f32 = 40.0;
const NAME_CELL: Vec2 = Vec2::new(72.0, 84.0);
const NAME_CELL_GAP: f32 = 24.0;
const NAME_LETTER_SIZE: f32 = 56.0;
const NAME_ARROW: f32 = 14.0;
const SCORES_PANEL_WIDTH: f32 = 680.0;
const SCORES_ROW_HEIGHT: f32 = 40.0;
const SCORES_TEXT_SIZE: f32 = 24.0;
// right edges of the rank, initials, score, level and lines columns
const SCORES_COLUMNS: [f32; 5] = [90.0, 210.0, 430.0, 530.0, 630.0];
const EMPTY_ROW_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.35);

#[derive(Debug)]
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
pub struct Meshes {
    rect: graphics::Mesh,            // unit rectangle, scaled and colored per draw
    background: graphics::Mesh,      // unit rectangle fading out downward, tinted per draw
    rounded: graphics::Mesh,         // BLOCK_SIZE rounded square, scaled per draw
    rounded_outline: graphics::Mesh, // jelly ghost cell
    glow: graphics::Mesh,            // unit disc fading to clear at the rim
}

impl Meshes {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
        let vertex = |x: f32, y: f32, color: Color| graphics::Vertex {
            position: [x, y],
            uv: [x, y],
            color: [color.r, color.g, color.b, color.a],
        };
        Ok(Self {
            rect: graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), unit, Color::WHITE)?,
            rounded: graphics::Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, BLOCK_SIZE, BLOCK_SIZE),
                JELLY_RADIUS,
                Color::WHITE,
            )?,
            rounded_outline: graphics::Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::stroke(GHOST_OUTLINE_WIDTH),
                inset(
                    Rect::new(0.0, 0.0, BLOCK_SIZE, BLOCK_SIZE),
                    GHOST_OUTLINE_WIDTH / 2.0,
                ),
                JELLY_RADIUS - GHOST_OUTLINE_WIDTH / 2.0,
                Color::WHITE,
            )?,
            background: graphics::Mesh::from_data(
                ctx,
                graphics::MeshData {
                    vertices: &[
                        vertex(0.0, 0.0, Color::WHITE),
                        vertex(1.0, 0.0, Color::WHITE),
                        vertex(1.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.0)),
                        vertex(0.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.0)),
                    ],
                    indices: &[0, 1, 2, 0, 2, 3],
                },
            ),
            glow: glow_mesh(ctx),
        })
    }
}

/// soft disc, rings of falling alpha
fn glow_mesh(ctx: &mut Context) -> graphics::Mesh {
    const RINGS: [(f32, f32); 4] = [(0.0, 1.0), (0.35, 0.6), (0.7, 0.2), (1.0, 0.0)];
    const SEGMENTS: u32 = 48;
    let mut vertices = Vec::new();
    for (radius, alpha) in RINGS {
        let count = if radius == 0.0 { 1 } else { SEGMENTS };
        for k in 0..count {
            let a = k as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (x, y) = (a.cos() * radius, a.sin() * radius);
            vertices.push(graphics::Vertex {
                position: [x, y],
                uv: [x, y],
                color: [1.0, 1.0, 1.0, alpha],
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
    graphics::Mesh::from_data(
        ctx,
        graphics::MeshData {
            vertices: &vertices,
            indices: &indices,
        },
    )
}

fn window_colors(style: BackgroundStyle) -> (Color, Color) {
    match style {
        BackgroundStyle::Stars => STARS_COLORS,
        BackgroundStyle::Plasma => PLASMA_COLORS,
        BackgroundStyle::Off => OFF_COLORS,
    }
}

/// window clear color for a background
pub fn clear_color(style: BackgroundStyle) -> Color {
    window_colors(style).1
}

/// vertical gradient for a background
pub fn draw_background(
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    style: BackgroundStyle,
) {
    let (top, bottom) = window_colors(style);
    draw_rect(canvas, meshes, *view_rect, bottom);
    canvas.draw(
        &meshes.background,
        graphics::DrawParam::default()
            .dest_rect(*view_rect)
            .color(top),
    );
}

/// gap line filling the snapped gap between blocks
fn snap_line(start: f32) -> (f32, f32) {
    let width = CELL - block_size();
    (snap(start + BLOCK_PADDING) - width, width)
}

fn draw_rect(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, color: Color) {
    canvas.draw(
        &meshes.rect,
        graphics::DrawParam::default().dest_rect(rect).color(color),
    );
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

/// text blends premultiplied
fn premultiply(color: Color) -> Color {
    Color::new(
        color.r * color.a,
        color.g * color.a,
        color.b * color.a,
        color.a,
    )
}

fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * amount,
        from.g + (to.g - from.g) * amount,
        from.b + (to.b - from.b) * amount,
        from.a + (to.a - from.a) * amount,
    )
}

fn shade(color: Color, amount: f32) -> Color {
    Color::new(
        color.r * amount,
        color.g * amount,
        color.b * amount,
        color.a,
    )
}

fn tint(color: Color, amount: f32) -> Color {
    Color::new(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

fn draw_rounded(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, color: Color) {
    canvas.draw(
        &meshes.rounded,
        graphics::DrawParam::default()
            .dest(Vec2::new(rect.x, rect.y))
            .scale(Vec2::new(rect.w / BLOCK_SIZE, rect.h / BLOCK_SIZE))
            .color(color),
    );
}

fn inset(rect: Rect, by: f32) -> Rect {
    Rect::new(
        rect.x + by,
        rect.y + by,
        rect.w - 2.0 * by,
        rect.h - 2.0 * by,
    )
}

/// rect edges of the given width, as non-overlapping rects
fn draw_outline(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, width: f32, color: Color) {
    let (x, y, w, h) = (rect.x, rect.y, rect.w, rect.h);
    for edge in [
        Rect::new(x, y, w, width),
        Rect::new(x, y + h - width, w, width),
        Rect::new(x, y + width, width, h - 2.0 * width),
        Rect::new(x + w - width, y + width, width, h - 2.0 * width),
    ] {
        draw_rect(canvas, meshes, edge, color);
    }
}

/// block in the selected style, edges scale with the block size
fn draw_block(canvas: &mut Canvas, meshes: &Meshes, rect: Rect, color: Color, style: BlockStyle) {
    let k = rect.w.min(rect.h) / BLOCK_SIZE;
    match style {
        BlockStyle::Bevel => {
            // light top left edge, dark bottom right edge and gloss
            let bevel = BEVEL * k;
            draw_rect(canvas, meshes, rect, shade(color, 0.55));
            let highlight = Rect::new(rect.x, rect.y, rect.w - bevel, rect.h - bevel);
            draw_rect(canvas, meshes, highlight, tint(color, 0.45));
            let face = inset(rect, bevel);
            draw_rect(canvas, meshes, face, color);
            let gloss = Rect::new(face.x, face.y, face.w, face.h * 0.45);
            draw_rect(
                canvas,
                meshes,
                gloss,
                with_alpha(Color::WHITE, 0.12 * color.a),
            );
        }
        BlockStyle::Flat => {
            draw_rect(canvas, meshes, rect, shade(color, 0.8));
            draw_rect(canvas, meshes, inset(rect, 2.0 * k), color);
        }
        BlockStyle::Neon => {
            // dim core with a bright rim
            draw_rect(
                canvas,
                meshes,
                rect,
                with_alpha(shade(color, 0.3), color.a * 0.85),
            );
            draw_outline(canvas, meshes, rect, NEON_EDGE * k, tint(color, 0.15));
            let core = Rect::new(
                rect.x + rect.w * 0.36,
                rect.y + rect.h * 0.36,
                rect.w * 0.28,
                rect.h * 0.28,
            );
            draw_rect(
                canvas,
                meshes,
                core,
                with_alpha(tint(color, 0.5), color.a * 0.7),
            );
        }
        BlockStyle::Retro => {
            // dark border, solid face and a pixel highlight in the corner
            let edge = RETRO_EDGE * k;
            let pixel = (rect.w.min(rect.h) / 7.5).round().max(1.0);
            draw_rect(canvas, meshes, rect, shade(color, 0.25));
            let face = inset(rect, edge);
            draw_rect(canvas, meshes, face, color);
            let shine = with_alpha(tint(color, 0.85), color.a);
            let (x, y) = (face.x + pixel, face.y + pixel);
            draw_rect(canvas, meshes, Rect::new(x, y, pixel, pixel), shine);
            draw_rect(
                canvas,
                meshes,
                Rect::new(x + pixel, y, pixel * 2.0, pixel),
                shine,
            );
            draw_rect(
                canvas,
                meshes,
                Rect::new(x, y + pixel, pixel, pixel * 2.0),
                shine,
            );
        }
        BlockStyle::Jelly => {
            // rounded with a soft shadow and a glossy top
            draw_rounded(canvas, meshes, rect, shade(color, 0.6));
            let face = Rect::new(rect.x, rect.y, rect.w, rect.h - 2.0 * k);
            draw_rounded(canvas, meshes, face, color);
            let gloss = Rect::new(
                face.x + face.w * 0.2,
                face.y + face.h * 0.12,
                face.w * 0.6,
                face.h * 0.3,
            );
            draw_rounded(
                canvas,
                meshes,
                gloss,
                with_alpha(Color::WHITE, 0.35 * color.a),
            );
        }
    }
}

/// piece color, grayed and flashed by the game over curtain at row y
fn block_color(rtype: RustominoType, game_over: Option<&GameOver>, y: i32) -> Color {
    let color = rtype.color();
    let Some(game_over) = game_over else {
        return color;
    };
    let gray = mix_color(color, util::rgb_to_grayscale(color), game_over.gray(y));
    mix_color(gray, Color::WHITE, CURTAIN_FLASH * game_over.flash(y))
}

/// fade out in a wipe from where the rustomino landed to the edges
fn line_clear_alpha(line_clear: &LineClear, x: usize) -> f32 {
    let start = line_clear.wipe_start(x);
    (1.0 - (line_clear.progress() - start) / playfield::LINE_CLEAR_FADE).clamp(0.0, 1.0) as f32
}

fn frame_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x - FRAME_WIDTH,
        rect.y - FRAME_WIDTH,
        rect.w + 2.0 * FRAME_WIDTH,
        rect.h + 2.0 * FRAME_WIDTH,
    )
}

fn draw_playing_backgound(
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_settings: &ViewSettings,
    board: BoardStyle,
    hold: bool,
) {
    // frames around the staging area and playfield, and the side panels
    let staging = view_settings.staging_rect;
    let playfield = view_settings.playfield_rect;
    let play_area = Rect::new(
        staging.x,
        staging.y,
        staging.w,
        playfield.y + playfield.h - staging.y,
    );
    let hold_rect = hold.then_some(view_settings.hold_rect);
    for rect in [Some(play_area), Some(view_settings.preview_rect), hold_rect]
        .into_iter()
        .flatten()
    {
        draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    }

    let backgrounds = [
        (view_settings.staging_rect, STAGING_BACKGROUND_COLOR),
        (view_settings.playfield_rect, PLAYFIELD_BACKGROUND_COLOR),
        (view_settings.preview_rect, PREVIEW_BACKGROUND_COLOR),
        (view_settings.hold_rect, HOLD_BACKGROUND_COLOR),
    ];
    for (rect, color) in backgrounds
        .into_iter()
        .filter(|(rect, _)| hold || *rect != view_settings.hold_rect)
    {
        draw_rect(canvas, meshes, rect, color);
    }

    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let (cols, rows) = (
        playfield::PLAYFIELD_SLOTS[0],
        playfield::PLAYFIELD_SIZE[1] as usize,
    );
    match board {
        BoardStyle::Grid => {
            // lines in the gaps between cells
            for col in 1..cols {
                let x = playfield.x + col as f32 * cell;
                let (x, w) = snap_line(x);
                let line = Rect::new(x, playfield.y, w, playfield.h);
                draw_rect(canvas, meshes, line, GRID_COLOR);
            }
            for row in 1..rows {
                let y = playfield.y + playfield.h - 2.0 * BLOCK_PADDING - row as f32 * cell;
                let (y, h) = snap_line(y);
                let line = Rect::new(playfield.x, y, playfield.w, h);
                draw_rect(canvas, meshes, line, GRID_COLOR);
            }
        }
        BoardStyle::Dots => {
            // dots where the gaps cross
            for row in 1..rows {
                let y = playfield.y + playfield.h - 2.0 * BLOCK_PADDING - row as f32 * cell;
                for col in 1..cols {
                    let x = playfield.x + col as f32 * cell;
                    let dot = snap_rect(Rect::new(
                        x + (BLOCK_PADDING - DOT_SIZE) / 2.0,
                        y + (BLOCK_PADDING - DOT_SIZE) / 2.0,
                        DOT_SIZE,
                        DOT_SIZE,
                    ));
                    draw_rect(canvas, meshes, dot, DOT_COLOR);
                }
            }
        }
        BoardStyle::Checker => {
            for row in 0..rows {
                for col in (row % 2..cols).step_by(2) {
                    let rect = playfield_block_rect(
                        [col as i32, row as i32],
                        &view_settings.staging_rect,
                        &playfield,
                    );
                    draw_rect(canvas, meshes, rect, CHECKER_COLOR);
                }
            }
        }
        BoardStyle::Clean => {}
    }
}

fn draw_playfield(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: BlockStyle,
    ghost: bool,
) {
    let (staging_rect, playfield_rect) =
        (&view_settings.staging_rect, &view_settings.playfield_rect);
    // draw the locked blocks
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        let clearing = playfield
            .line_clear
            .as_ref()
            .filter(|line_clear| line_clear.lines.contains(&y));
        for (x, slot) in slots_x.iter().enumerate() {
            if let SlotState::Locked(rtype) = slot {
                let mut color = block_color(*rtype, game_over, y as i32);
                if let Some(line_clear) = clearing {
                    let progress = line_clear.progress() as f32;
                    let flash = (1.0 - progress * 4.0).max(0.0);
                    let base = if line_clear.lines.len() >= 4 {
                        // glint riding the wipe outward
                        let bottom = line_clear.lines.iter().min().copied().unwrap_or(y);
                        let peak = line_clear.wipe_start(x) as f32
                            + QUAD_GLINT_LEAD
                            + (y - bottom) as f32 * QUAD_GLINT_ROW;
                        let glint = (1.0 - (progress - peak).abs() / QUAD_GLINT_WIDTH).max(0.0);
                        mix_color(LINE_CLEAR_COLOR, Color::WHITE, glint * 0.8)
                    } else {
                        LINE_CLEAR_COLOR
                    };
                    color = mix_color(base, Color::WHITE, flash);
                    color.a *= line_clear_alpha(line_clear, x);
                }
                let rect = playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                draw_block(canvas, meshes, rect, color, style);
            }
        }
    }

    // draw the active rustomino over the locked blocks
    if let Some(active) = &playfield.active_rustomino {
        for block in active.playfield_slots() {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            let color = block_color(active.rtype, game_over, block[1]);
            draw_block(canvas, meshes, rect, color, style);
        }
    }

    if let Some(ghost) = playfield.ghost_rustomino.as_ref().filter(|_| ghost) {
        let color = block_color(ghost.rtype, game_over, PREVIEW_ROW);
        let slots = ghost.playfield_slots();
        if style == BlockStyle::Jelly {
            // rounded cells like the blocks
            for block in &slots {
                let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
                draw_rounded(canvas, meshes, rect, with_alpha(color, GHOST_FILL_ALPHA));
                let param = graphics::DrawParam::default()
                    .dest(Vec2::new(rect.x, rect.y))
                    .scale(Vec2::new(rect.w / BLOCK_SIZE, rect.h / BLOCK_SIZE))
                    .color(with_alpha(color, GHOST_OUTLINE_ALPHA));
                canvas.draw(&meshes.rounded_outline, param);
            }
            return;
        }
        for block in &slots {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            draw_rect(canvas, meshes, rect, with_alpha(color, GHOST_FILL_ALPHA));
        }
        let outline = with_alpha(color, GHOST_OUTLINE_ALPHA);
        for rect in ghost_outline(&slots, staging_rect, playfield_rect) {
            draw_rect(canvas, meshes, rect, outline);
        }
    }
}

/// fixed width outline inside the ghost piece's boundary, as non-overlapping rects
fn ghost_outline(slots: &[IVec2], staging_rect: &Rect, playfield_rect: &Rect) -> Vec<Rect> {
    let s = block_size();
    let (p, w) = (CELL - s, snap(GHOST_OUTLINE_WIDTH));
    let has = |x: i32, y: i32| slots.contains(&IVec2::new(x, y));
    let mut rects = Vec::new();
    for &IVec2 { x, y } in slots {
        let r = playfield_block_rect([x, y], staging_rect, playfield_rect);
        // slot y+1 is above on screen
        let (l, rt, u, d) = (has(x - 1, y), has(x + 1, y), has(x, y + 1), has(x, y - 1));
        let (ul, ur, dl, dr) = (
            has(x - 1, y + 1),
            has(x + 1, y + 1),
            has(x - 1, y - 1),
            has(x + 1, y - 1),
        );

        // sides, verticals trimmed where they meet horizontals
        if !u {
            rects.push(Rect::new(r.x, r.y, s, w));
        }
        if !d {
            rects.push(Rect::new(r.x, r.y + s - w, s, w));
        }
        let top = if u { 0.0 } else { w };
        let side_h = s - top - if d { 0.0 } else { w };
        if !l {
            rects.push(Rect::new(r.x, r.y + top, w, side_h));
        }
        if !rt {
            rects.push(Rect::new(r.x + s - w, r.y + top, w, side_h));
        }

        // inner corners
        if l && u && !ul {
            rects.push(Rect::new(r.x, r.y, w, w));
        }
        if rt && u && !ur {
            rects.push(Rect::new(r.x + s - w, r.y, w, w));
        }
        if l && d && !dl {
            rects.push(Rect::new(r.x, r.y + s - w, w, w));
        }
        if rt && d && !dr {
            rects.push(Rect::new(r.x + s - w, r.y + s - w, w, w));
        }

        // bridge the padding to the right and above
        if rt {
            if !(u && ur) {
                rects.push(Rect::new(r.x + s, r.y, p, w));
            }
            if !(d && dr) {
                rects.push(Rect::new(r.x + s, r.y + s - w, p, w));
            }
        }
        if u {
            if !(l && ul) {
                rects.push(Rect::new(r.x, r.y - p, w, p));
            }
            if !(rt && ur) {
                rects.push(Rect::new(r.x + s - w, r.y - p, w, p));
            }
        }
    }
    rects
}

fn draw_preview(
    canvas: &mut Canvas,
    meshes: &Meshes,
    rustomino: &Option<Rustomino>,
    preview_rect: &Rect,
    game_over: Option<&GameOver>,
    style: BlockStyle,
) {
    if let Some(rustomino) = rustomino {
        // center the piece's bounding box in the panel
        let cell = BLOCK_SIZE + BLOCK_PADDING;
        let min = rustomino
            .blocks
            .iter()
            .fold(rustomino.blocks[0], |a, b| a.min(*b));
        let max = rustomino
            .blocks
            .iter()
            .fold(rustomino.blocks[0], |a, b| a.max(*b));
        let width = (max.x - min.x + 1) as f32 * cell - BLOCK_PADDING;
        let height = (max.y - min.y + 1) as f32 * cell - BLOCK_PADDING;
        let left = snap(preview_rect.x + (preview_rect.w - width) / 2.0);
        let top = snap(preview_rect.y + (preview_rect.h - height) / 2.0);
        for block in rustomino.blocks {
            let rect = Rect::new(
                left + (block.x - min.x) as f32 * cell,
                top + (max.y - block.y) as f32 * cell,
                block_size(),
                block_size(),
            );
            draw_block(
                canvas,
                meshes,
                rect,
                block_color(rustomino.rtype, game_over, PREVIEW_ROW),
                style,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_playing(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    next_rustomino: &Option<Rustomino>,
    hold_rustomino: &Option<Rustomino>,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: &Style,
    assists: bool, // ghost and hold
) {
    draw_playing_backgound(canvas, meshes, view_settings, style.board, assists);
    draw_playfield(
        canvas,
        meshes,
        playfield,
        view_settings,
        game_over,
        style.block,
        assists,
    );
    if assists {
        draw_preview(
            canvas,
            meshes,
            hold_rustomino,
            &view_settings.hold_rect,
            game_over,
            style.block,
        );
    }
    draw_preview(
        canvas,
        meshes,
        next_rustomino,
        &view_settings.preview_rect,
        game_over,
        style.block,
    );
}

// physical pixels per view unit, text is rasterized at the physical size
static PIXEL_SCALE: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub fn set_pixel_scale(scale: f32) {
    PIXEL_SCALE.store(scale.to_bits(), Ordering::Relaxed);
}

fn pixel_scale() -> f32 {
    f32::from_bits(PIXEL_SCALE.load(Ordering::Relaxed))
}

/// to the nearest whole physical pixel
fn snap(v: f32) -> f32 {
    let scale = pixel_scale();
    (v * scale).round() / scale
}

/// position and size snapped, so moving rects keep their size
fn snap_rect(rect: Rect) -> Rect {
    Rect::new(snap(rect.x), snap(rect.y), snap(rect.w), snap(rect.h))
}

/// edges snapped, so adjacent rects still meet
fn snap_edges(rect: Rect) -> Rect {
    let (x, y) = (snap(rect.x), snap(rect.y));
    Rect::new(x, y, snap(rect.x + rect.w) - x, snap(rect.y + rect.h) - y)
}

/// block size in whole pixels, with at least a pixel of gap
fn block_size() -> f32 {
    let scale = pixel_scale();
    let cell = (CELL * scale).round();
    (BLOCK_SIZE * scale).round().min(cell - 1.0).max(1.0) / scale
}

fn px(size: f32) -> graphics::PxScale {
    graphics::PxScale::from(size * pixel_scale())
}

fn text_param() -> graphics::DrawParam {
    graphics::DrawParam::default().scale(Vec2::splat(1.0 / pixel_scale()))
}

trait MeasureUi {
    /// size in view units
    fn measure_ui(&self, ctx: &Context) -> GameResult<Vec2>;
}

impl MeasureUi for graphics::Text {
    fn measure_ui(&self, ctx: &Context) -> GameResult<Vec2> {
        Ok(Vec2::from(self.measure(ctx)?) / pixel_scale())
    }
}

fn ui_text(text: &str, size: f32) -> graphics::Text {
    let mut text = graphics::Text::new(text);
    text.set_font("04b30").set_scale(px(size));
    text
}

fn draw_shadowed(canvas: &mut Canvas, text: &graphics::Text, pos: Vec2, color: Color) {
    let param = text_param();
    // the shadow fades with the text
    let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * color.a);
    canvas.draw(text, param.dest(pos + Vec2::new(2.0, 2.0)).color(shadow));
    canvas.draw(text, param.dest(pos).color(premultiply(color)));
}

/// text in the ui font with a drop shadow
fn draw_text(canvas: &mut Canvas, text: &str, pos: Vec2, size: f32, color: Color) {
    draw_shadowed(canvas, &ui_text(text, size), pos, color);
}

/// text positioned by its center
fn draw_text_centered(
    ctx: &Context,
    canvas: &mut Canvas,
    text: &graphics::Text,
    center: Vec2,
    color: Color,
) -> GameResult {
    let size = text.measure_ui(ctx)?;
    draw_shadowed(
        canvas,
        text,
        center - Vec2::new(size.x / 2.0, size.y / 2.0),
        color,
    );
    Ok(())
}

pub fn draw_playing_text(
    _ctx: &mut Context,
    canvas: &mut Canvas,
    level: &ScoreDisplay,
    score: &ScoreDisplay,
    best: usize,
    view_settings: &ViewSettings,
    hold_shown: bool,
) -> GameResult {
    let label_size = UI_FONT_SIZE * 0.75;
    let label_offset = Vec2::new(0.0, -(label_size + FRAME_WIDTH + 8.0));
    let hold = view_settings.hold_rect;
    let preview = view_settings.preview_rect;

    draw_piece_colored(canvas, "Blocks!", view_settings.title_pos, UI_FONT_SIZE);
    if hold_shown {
        draw_text(
            canvas,
            "Hold",
            Vec2::new(hold.x, hold.y) + label_offset,
            label_size,
            LABEL_COLOR,
        );
    }
    draw_text(
        canvas,
        "Next",
        Vec2::new(preview.x, preview.y) + label_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Level:",
        view_settings.level_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Score:",
        view_settings.score_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_counter(canvas, level, view_settings.level_pos);
    draw_counter(canvas, score, view_settings.score_pos);
    let best_offset = Vec2::new(0.0, -BEST_OFFSET);
    draw_text(
        canvas,
        "Best:",
        view_settings.score_label_pos + best_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        &best.to_string(),
        view_settings.score_pos + best_offset,
        label_size,
        Color::WHITE,
    );

    Ok(())
}

/// value that counts up and pops when it increases, with rising "+amount"
fn draw_counter(canvas: &mut Canvas, score: &ScoreDisplay, pos: Vec2) {
    let pop = score.pop();
    let size = UI_FONT_SIZE * (1.0 + SCORE_POP_SCALE * pop);
    let grow = Vec2::new(0.0, (size - UI_FONT_SIZE) / 2.0);
    let color = mix_color(Color::WHITE, ACCENT_COLOR, pop);
    draw_text(canvas, &score.shown().to_string(), pos - grow, size, color);

    for popup in &score.popups {
        let t = popup.progress();
        let rise = Vec2::new(0.0, SCORE_POPUP_OFFSET + SCORE_POPUP_RISE * t);
        let color = with_alpha(ACCENT_COLOR, 1.0 - t * t);
        let text = format!("+{}", popup.points);
        draw_text(canvas, &text, pos - rise, UI_FONT_SIZE * 0.8, color);
    }
}

/// stable pseudo random value in 0.0..1.0 for item i
fn hash(i: usize, n: f32) -> f32 {
    ((i as f32 * 12.9898 + n * 78.233).sin() * 43758.547)
        .fract()
        .abs()
}

fn smoothstep(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// level up ring strength at a point, 0.0 to 1.0
fn level_ring(background: &Background, view_rect: &Rect, point: Vec2) -> (f32, Vec2) {
    let Some(p) = background.wave() else {
        return (0.0, Vec2::ZERO);
    };
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let reach = center.distance(view_rect.point().into()) + RING_WIDTH * 2.0;
    let radius = (1.0 - (1.0 - p).powi(2)) * reach;
    let offset = point - center;
    let x = (offset.length() - radius) / RING_WIDTH;
    ((-x * x).exp() * (1.0 - p), offset.normalize_or_zero())
}

/// drain a color to icy gray
fn frost(color: Color, amount: f32) -> Color {
    let ice = mix_color(util::rgb_to_grayscale(color), FROST_COLOR, 0.4);
    mix_color(color, with_alpha(ice, color.a), amount)
}

fn overlaps(a: Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/// animated background for the selected style
pub fn draw_background_animation(
    canvas: &mut Canvas,
    meshes: &Meshes,
    background: &Background,
    view_rect: &Rect,
) {
    match background.style {
        BackgroundStyle::Stars => draw_stars(canvas, meshes, view_rect, background),
        BackgroundStyle::Plasma => draw_plasma(canvas, meshes, view_rect, background),
        BackgroundStyle::Off => {}
    }
}

struct Star {
    pos: Vec2,
    size: f32,
    depth: f32, // 0.0 far to 1.0 near
    alpha: f32,
    color: Color,
}

/// star i of a field, flying toward the camera
fn star_of(i: usize, field: usize, background: &Background, view_rect: &Rect) -> Star {
    let n = |k: f32| hash(i, k + field as f32 * 10.0);
    let time = background.time;
    let z = (n(1.0) - time * STAR_FLIGHT).rem_euclid(1.0).max(STAR_NEAR);
    let depth = 1.0 - z;
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let spread = Vec2::new(n(2.0) * 2.0 - 1.0, n(3.0) * 2.0 - 1.0) * STAR_SPREAD / z;
    let pos = center + spread * Vec2::new(view_rect.w, view_rect.h) / 2.0;
    let size = (1.0 + 3.0 * depth * depth).round();
    // twinkle deepens and some stars sparkle once stopped
    let chill = background.chill();
    let clock = background.twinkle;
    let wave = (clock * (2.0 + 3.0 * n(4.0)) + n(5.0) * std::f32::consts::TAU).sin();
    let swing = 0.4 + (STARS_STILL_TWINKLE - 0.4) * chill;
    let twinkle = 1.0 - swing + swing * wave;
    let sparkle = if n(7.0) < STARS_SPARKLE_CHANCE {
        let at = (clock / STARS_SPARKLE_PERIOD + n(8.0)).fract();
        (1.0 - (at * STARS_SPARKLE_PERIOD / STARS_SPARKLE_TIME).min(1.0)).powi(2) * chill
    } else {
        0.0
    };
    // fade in far away
    let arrive = smoothstep(1.0, 0.8, z);
    let alpha = ((0.15 + 0.45 * depth) * twinkle * (1.0 + background.energy)
        + STARS_SPARKLE * sparkle)
        .min(1.0)
        * arrive;
    let size = size + (sparkle * STARS_SPARKLE * 2.0).round();
    let color = if n(6.0) < 0.15 {
        RustominoType::iter()
            .nth(i % 7)
            .map_or(STAR_COLOR, |t| t.color())
    } else {
        STAR_COLOR
    };
    Star {
        pos: pos - Vec2::splat(size / 2.0),
        size,
        depth,
        alpha,
        color,
    }
}

fn draw_stars(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect, background: &Background) {
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let field = background.stars;
    let Some(p) = background.wave() else {
        draw_nebula(canvas, meshes, view_rect, background, field, 1.0, 1.0);
        for i in 0..STAR_COUNT {
            let star = star_of(i, field, background, view_rect);
            let rect = Rect::new(star.pos.x, star.pos.y, star.size, star.size);
            if star.alpha > 0.0 && overlaps(rect, view_rect) {
                draw_rect(canvas, meshes, rect, with_alpha(star.color, star.alpha));
            }
        }
        return;
    };
    let old = (field + STAR_FIELDS - 1) % STAR_FIELDS;
    let appear = smoothstep(WARP_ARRIVE, 1.0, p);
    // old nebula rushes past, the next fades in behind it
    let leave = 1.0 - smoothstep(0.1, 0.7, p);
    draw_nebula(
        canvas,
        meshes,
        view_rect,
        background,
        old,
        leave,
        1.0 + smoothstep(0.0, 0.7, p),
    );
    draw_nebula(
        canvas,
        meshes,
        view_rect,
        background,
        field,
        appear,
        0.8 + 0.2 * appear,
    );
    for i in 0..STAR_COUNT {
        // old field streaks out toward the camera
        let star = star_of(i, old, background, view_rect);
        draw_warp_star(canvas, meshes, view_rect, center, &star, p);
        // new field arrives from behind, small and dim, spreading into place
        if appear <= 0.0 {
            continue;
        }
        let star = star_of(i, field, background, view_rect);
        let pos = center + (star.pos - center) * (0.7 + 0.3 * appear);
        let size = (star.size * appear).round().max(1.0);
        let rect = Rect::new(pos.x, pos.y, size, size);
        if overlaps(rect, view_rect) {
            draw_rect(
                canvas,
                meshes,
                rect,
                with_alpha(star.color, star.alpha * appear),
            );
        }
    }
}

/// soft clouds behind a star field, drawn additively
fn draw_nebula(
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    background: &Background,
    field: usize,
    alpha: f32,
    zoom: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let view = Vec2::new(view_rect.w, view_rect.h);
    let span = view_rect.w.max(view_rect.h);
    let drift = background.time * NEBULA_DRIFT;
    canvas.set_blend_mode(graphics::BlendMode::ADD);
    for cloud in 0..NEBULA_CLOUDS {
        let n = |k: f32| hash(cloud, k + field as f32 * 10.0 + 500.0);
        let hue = NEBULA_COLORS[(n(1.0) * NEBULA_COLORS.len() as f32) as usize];
        let edge = NEBULA_COLORS[(n(2.0) * NEBULA_COLORS.len() as f32) as usize];
        let at = Vec2::new(n(3.0) - 0.5, n(4.0) - 0.5) * view * 1.1;
        let size = span * (0.18 + 0.12 * n(5.0));
        for blob in 0..NEBULA_BLOBS {
            let m = |k: f32| hash(cloud * NEBULA_BLOBS + blob, k + field as f32 * 10.0 + 700.0);
            let angle = m(1.0) * std::f32::consts::TAU;
            let out = Vec2::new(angle.cos(), angle.sin()) * size * m(2.0);
            // slow swirl
            let phase = m(3.0) * std::f32::consts::TAU;
            let sway = Vec2::new((drift + phase).sin(), (drift * 0.7 + phase).cos()) * size * 0.08;
            let pos = center + (at + out + sway) * zoom;
            let radius = size * (0.5 + 0.6 * m(4.0)) * zoom;
            let stretch = 0.45 + 0.55 * m(5.0);
            let color = mix_color(hue, edge, m(6.0));
            let strength = NEBULA_ALPHA * (0.5 + 0.5 * m(7.0)) * alpha;
            canvas.draw(
                &meshes.glow,
                graphics::DrawParam::default()
                    .dest(pos)
                    .rotation(m(8.0) * std::f32::consts::PI + drift * 0.1)
                    .scale(Vec2::new(radius, radius * stretch))
                    .color(with_alpha(color, strength)),
            );
        }
    }
    canvas.set_blend_mode(graphics::BlendMode::ALPHA);
}

/// a star streaking out from the center, head first, tail catching up
fn draw_warp_star(
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    center: Vec2,
    star: &Star,
    p: f32,
) {
    // streaks only head outward, so off screen stars stay off
    if !overlaps(
        Rect::new(star.pos.x, star.pos.y, star.size, star.size),
        view_rect,
    ) {
        return;
    }
    let reach = (star.pos - center) * WARP_STREAK * star.depth;
    let head = reach * smoothstep(0.0, 0.45, p);
    let tail = reach * smoothstep(0.2, 0.65, p);
    // grows as it nears
    let size = (star.size * (1.0 + smoothstep(0.0, 0.65, p))).round();
    let length = (head - tail).length();
    let dots = ((length / size) as usize).clamp(1, WARP_DOTS);
    let bright =
        (star.alpha * (1.0 + smoothstep(0.0, 0.3, p))).min(1.0) * (1.0 - smoothstep(0.4, 0.65, p));
    for d in 0..=dots {
        let f = d as f32 / dots as f32;
        let at = star.pos + tail + (head - tail) * f;
        let dot = Rect::new(at.x, at.y, size, size);
        if overlaps(dot, view_rect) {
            let fade = if dots == 1 { 1.0 } else { 0.3 + 0.7 * f };
            draw_rect(canvas, meshes, dot, with_alpha(star.color, bright * fade));
        }
    }
}

fn draw_plasma(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect, background: &Background) {
    let glow = 1.0 + background.energy;
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    // cells counted from the view center so resizing doesn't shift the pattern
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let cols = (view_rect.w / 2.0 / PLASMA_CELL).ceil() as i32;
    let rows = (view_rect.h / 2.0 / PLASMA_CELL).ceil() as i32;
    let frozen = background.frozen();
    let seeds: Vec<Vec2> = (0..CRYSTAL_SEEDS)
        .map(|i| {
            let n = |k: f32| hash(i, k + background.stars as f32 * 10.0);
            Vec2::new(
                (n(1.0) * 2.0 - 1.0) * cols as f32,
                (n(2.0) * 2.0 - 1.0) * rows as f32,
            )
        })
        .collect();
    for row in -rows..rows {
        for col in -cols..cols {
            let (x, y) = (col as f32, row as f32);
            let cell = ((row + 1000) * 2000 + col + 1000) as usize;
            // crystals grow from seeds, cells stop as they freeze
            let ice = frozen.map(|t| {
                let grow = seeds
                    .iter()
                    .map(|s| (s.x - x).abs() + (s.y - y).abs())
                    .fold(f32::MAX, f32::min);
                let at = grow * CRYSTAL_GROW + hash(cell, 1.0) * CRYSTAL_JITTER;
                (t - at, at)
            });
            let pattern = |time: f32| {
                let t = time * PLASMA_SPEED;
                let ring = (x * x + y * y).sqrt();
                let v = (x * 0.35 + t * 1.1).sin()
                    + (y * 0.3 - t * 0.8).sin()
                    + ((x + y) * 0.18 + t * 0.6).sin()
                    + (ring * 0.4 - t * 1.5).sin();
                let n = (v + 4.0) / 8.0;
                (n, (n * 10.0 + t * 0.3) as usize)
            };
            let pos = center + Vec2::new(x, y) * PLASMA_CELL;
            let rect = Rect::new(pos.x, pos.y, PLASMA_CELL - 2.0, PLASMA_CELL - 2.0);
            let mut melt = 1.0;
            if let Some((since, at)) = ice.filter(|(since, _)| *since >= 0.0) {
                // faceted ice, flash as it forms, rare glints
                let (n, index) = pattern(background.frozen_time(at));
                let color = colors[index % colors.len()];
                let facet = hash(cell, 2.0);
                let glint_at = (since + hash(cell, 3.0) * GLINT_PERIOD) % GLINT_PERIOD;
                let glint = if hash(cell, 4.0) < GLINT_CHANCE {
                    1.0 - smoothstep(0.0, GLINT_TIME, glint_at)
                } else {
                    0.0
                };
                let flash = 1.0 - smoothstep(0.0, CRYSTAL_FLASH, since);
                let ice = mix_color(frost(color, 1.0), FROST_COLOR, 0.5);
                let ice = mix_color(ice, Color::WHITE, flash.max(glint));
                // melts in the order it froze
                let thaw = at * MELT_ORDER;
                melt = smoothstep(thaw, thaw + MELT_FADE, background.thawed());
                let alpha =
                    (0.008 + 0.02 * n * n + 0.015 * facet) * glow + 0.06 * flash + 0.08 * glint;
                draw_rect(canvas, meshes, rect, with_alpha(ice, alpha * (1.0 - melt)));
                if melt <= 0.0 {
                    continue;
                }
            }
            let (n, index) = pattern(background.time);
            let (wave, _) = level_ring(background, view_rect, pos + Vec2::splat(PLASMA_CELL / 2.0));
            // the ring shifts colors as it passes
            let shift = (wave * 2.0) as usize;
            let color = colors[(index + shift) % colors.len()];
            let color = mix_color(color, Color::WHITE, wave * 0.3);
            let alpha = ((0.01 + 0.04 * n * n) * glow + 0.15 * wave).min(1.0);
            draw_rect(canvas, meshes, rect, with_alpha(color, alpha * melt));
        }
    }
}

pub fn draw_overlay(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect) {
    draw_overlay_alpha(canvas, meshes, view_rect, 1.0);
}

fn draw_overlay_alpha(canvas: &mut Canvas, meshes: &Meshes, view_rect: &Rect, alpha: f32) {
    let color = with_alpha(PAUSED_OVERLAY_COLOR, PAUSED_OVERLAY_COLOR.a * alpha);
    draw_rect(canvas, meshes, *view_rect, color);
}

/// large wobbling title, optionally with each letter in a piece color
/// text with each letter colored like a piece, and a matching shadow text
/// per letter colors override the draw color, so the shadow is its own text
fn piece_colored_text(text: &str, size: f32) -> (graphics::Text, graphics::Text) {
    piece_colored_text_alpha(text, size, 1.0)
}

fn piece_colored_text_alpha(text: &str, size: f32, alpha: f32) -> (graphics::Text, graphics::Text) {
    let colors: Vec<Color> = RustominoType::iter()
        .map(|t| with_alpha(t.color(), alpha))
        .collect();
    let letters = |color_at: &dyn Fn(usize) -> Color| {
        let mut letters = graphics::Text::default();
        for (i, c) in text.chars().enumerate() {
            letters.add(TextFragment::new(c).color(color_at(i)));
        }
        letters.set_font("04b30").set_scale(px(size));
        letters
    };
    (
        letters(&|i| premultiply(tint(colors[i % colors.len()], TITLE_TINT))),
        letters(&|_| with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * alpha)),
    )
}

fn draw_piece_colored(canvas: &mut Canvas, text: &str, pos: Vec2, size: f32) {
    let (text, shadow) = piece_colored_text(text, size);
    let param = text_param();
    canvas.draw(
        &shadow,
        param.dest(pos + Vec2::splat((size / 16.0).max(2.0))),
    );
    canvas.draw(&text, param.dest(pos));
}

/// large wobbling piece colored title
fn draw_title(
    ctx: &Context,
    canvas: &mut Canvas,
    title: &str,
    center_y: f32,
    size: f32,
    view_rect: &Rect,
) -> GameResult {
    let wobble = util::slow_wobble(ctx.time.time_since_start().as_secs_f32());
    let center = Vec2::new(view_rect.w / 2.0, center_y + wobble * size / 12.0);
    let dims = piece_colored_text(title, size).0.measure_ui(ctx)?;
    let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    draw_piece_colored(canvas, title, pos, size);
    Ok(())
}

fn draw_hint(ctx: &Context, canvas: &mut Canvas, hint: &str, view_rect: &Rect) -> GameResult {
    let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
    draw_text_centered(
        ctx,
        canvas,
        &ui_text(hint, SMALL_TEXT_SIZE),
        center,
        LABEL_COLOR,
    )
}

fn draw_panel(canvas: &mut Canvas, meshes: &Meshes, rect: Rect) {
    draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    draw_rect(canvas, meshes, rect, PANEL_COLOR);
}

fn menu_panel_rect(view_rect: &Rect, rows: usize, top: f32, width: f32, row_height: f32) -> Rect {
    let height = rows as f32 * row_height + 2.0 * PANEL_PADDING;
    Rect::new(view_rect.w / 2.0 - width / 2.0, top, width, height)
}

fn menu_row_rect(panel: Rect, row: usize, row_height: f32) -> Rect {
    Rect::new(
        panel.x + PANEL_PADDING,
        panel.y + PANEL_PADDING + row as f32 * row_height,
        panel.w - 2.0 * PANEL_PADDING,
        row_height,
    )
}

/// highlight bar with a bobbing gold block at each end
fn draw_selection(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    row: Rect,
    style: BlockStyle,
) {
    draw_rect(canvas, meshes, row, HIGHLIGHT_COLOR);
    let bob = util::fast_wobble(ctx.time.time_since_start().as_secs_f32()) * 3.0;
    let size = row.h * 0.3;
    let y = row.y + row.h / 2.0 - size / 2.0;
    for x in [row.x + 12.0 + bob, row.x + row.w - 12.0 - size - bob] {
        draw_block(
            canvas,
            meshes,
            Rect::new(x, y, size, size),
            ACCENT_COLOR,
            style,
        );
    }
}

fn draw_menu_list<T: Menu>(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &T,
    panel: Rect,
    style: BlockStyle,
) -> GameResult {
    for (i, item) in menu.items().iter().enumerate() {
        let row = menu_row_rect(panel, i, MENU_ROW_HEIGHT);
        let selected = menu.selected() == i;
        if selected {
            draw_selection(ctx, canvas, meshes, row, style);
        }
        let mut text = item.clone();
        text.set_font("04b30").set_scale(px(MENU_ITEM_SIZE));
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
    }
    Ok(())
}

pub fn draw_menu(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    view_settings: &ViewSettings,
    style: BlockStyle,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_title(
        ctx,
        canvas,
        "Blocks!",
        view_rect.h * 0.16,
        TITLE_SIZE,
        view_rect,
    )?;
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.42,
        MENU_PANEL_WIDTH,
        MENU_ROW_HEIGHT,
    );
    draw_panel(canvas, meshes, panel);
    draw_menu_list(ctx, canvas, meshes, menu, panel, style)?;
    draw_hint(ctx, canvas, "Up/Down: select   Enter: choose", view_rect)
}

pub fn draw_paused(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    view_settings: &ViewSettings,
    style: BlockStyle,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_overlay(canvas, meshes, view_rect);
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.36,
        MENU_PANEL_WIDTH,
        MENU_ROW_HEIGHT,
    );
    let title_y = panel.y - PAUSED_TITLE_SIZE * 0.75;
    draw_title(ctx, canvas, "Paused", title_y, PAUSED_TITLE_SIZE, view_rect)?;
    draw_panel(canvas, meshes, panel);
    draw_menu_list(ctx, canvas, meshes, menu, panel, style)?;
    draw_hint(
        ctx,
        canvas,
        "Up/Down: select   Enter: choose   Esc: resume",
        view_rect,
    )
}

/// segments under the playfield filled per line toward the next level
pub fn draw_level_progress(
    canvas: &mut Canvas,
    meshes: &Meshes,
    progress: &LevelProgress,
    view_settings: &ViewSettings,
    style: BlockStyle,
) {
    let bar = view_settings.progress_rect;
    let burst = progress.burst();
    let frame_color = mix_color(FRAME_COLOR, ACCENT_COLOR, burst);
    draw_rect(canvas, meshes, frame_rect(bar), frame_color);
    draw_rect(canvas, meshes, bar, PLAYFIELD_BACKGROUND_COLOR);

    // whole pixel segments and gaps, centered in the bar
    let per_level = progress.len();
    let gap = snap(BLOCK_PADDING).max(1.0 / pixel_scale());
    let pitch = snap((bar.w - gap) / per_level as f32);
    let left = bar.x + snap((bar.w - gap - pitch * per_level as f32) / 2.0) + gap;
    let (top, height) = (snap(bar.y + BLOCK_PADDING), snap(PROGRESS_HEIGHT));
    let center_y = top + height / 2.0;
    let pulse = progress.pulse();
    for i in 0..per_level {
        let rect = Rect::new(left + i as f32 * pitch, top, pitch - gap, height);
        draw_rect(canvas, meshes, rect, SLIDER_EMPTY_COLOR);
        if let Some((_, amount)) = pulse.filter(|(at, _)| *at == i) {
            draw_rect(
                canvas,
                meshes,
                rect,
                with_alpha(ACCENT_COLOR, amount * 0.35),
            );
        }
        let Some(look) = progress.segment(i) else {
            continue;
        };
        let (w, h) = (rect.w * look.width, rect.h * look.height);
        let sized = snap_rect(Rect::new(
            rect.center().x - w / 2.0,
            center_y - h / 2.0,
            w,
            h,
        ));
        // soft two layer halo
        for (spread, alpha) in [(4.0, 0.12), (2.0, 0.22)] {
            if look.glow <= 0.0 {
                break;
            }
            let spread = spread * (0.5 + look.glow);
            let halo = Rect::new(
                sized.x - spread,
                sized.y - spread,
                sized.w + 2.0 * spread,
                sized.h + 2.0 * spread,
            );
            draw_rect(
                canvas,
                meshes,
                snap_rect(halo),
                with_alpha(ACCENT_COLOR, look.glow * alpha),
            );
        }
        let color = mix_color(ACCENT_COLOR, Color::WHITE, look.heat);
        draw_block(canvas, meshes, sized, color, style);
    }

    for spark in &progress.sparks {
        let size = snap(spark.size);
        let x = left + spark.pos.x * pitch - gap / 2.0 - size / 2.0;
        let y = center_y + spark.pos.y - size / 2.0;
        let color = if spark.white {
            Color::WHITE
        } else {
            ACCENT_COLOR
        };
        draw_rect(
            canvas,
            meshes,
            snap_rect(Rect::new(x, y, size, size)),
            with_alpha(color, spark.fade()),
        );
    }
}

/// row of blocks filled up to the level, right aligned to right_x
fn draw_slider(
    canvas: &mut Canvas,
    meshes: &Meshes,
    right_x: f32,
    center_y: f32,
    level: u8,
    max: u8,
    style: BlockStyle,
) {
    let (segment, pitch) = (snap(SLIDER_SEGMENT), snap(SLIDER_SEGMENT + SLIDER_GAP));
    let left = snap(right_x - max as f32 * pitch + pitch - segment);
    for i in 0..max {
        let rect = Rect::new(
            left + i as f32 * pitch,
            snap(center_y - segment / 2.0),
            segment,
            segment,
        );
        if i < level {
            draw_block(canvas, meshes, rect, ACCENT_COLOR, style);
        } else {
            draw_rect(canvas, meshes, rect, SLIDER_EMPTY_COLOR);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_options(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    menu: &menus::MenuList,
    mode: Mode,
    mode_locked: bool, // during a game
    theme: &str,
    background: &str,
    style: &Style,
    music_level: u8,
    sound_level: u8,
    max_level: u8,
    view_settings: &ViewSettings,
) -> GameResult {
    let view_rect = &view_settings.view_rect;
    draw_title(
        ctx,
        canvas,
        "Options",
        view_rect.h * 0.12,
        TITLE_SIZE,
        view_rect,
    )?;

    // theme, volume sliders and back
    let panel = menu_panel_rect(
        view_rect,
        menu.items().len(),
        view_rect.h * 0.19,
        OPTIONS_PANEL_WIDTH,
        OPTION_ROW_HEIGHT,
    );
    let slider_width = max_level as f32 * (SLIDER_SEGMENT + SLIDER_GAP) - SLIDER_GAP;
    draw_panel(canvas, meshes, panel);
    for (i, item) in menu.items().iter().enumerate() {
        let row = menu_row_rect(panel, i, OPTION_ROW_HEIGHT);
        let selected = menu.selected() == i;
        if selected {
            draw_selection(ctx, canvas, meshes, row, style.block);
        }
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        let mut text = item.clone();
        text.set_font("04b30");
        if i == menus::OPTION_BACK {
            text.set_scale(px(MENU_ITEM_SIZE));
            draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
            continue;
        }

        // label on the left, setting on the right
        text.set_scale(px(OPTION_LABEL_SIZE));
        let size = text.measure_ui(ctx)?;
        let center_y = row.y + row.h / 2.0;
        draw_shadowed(
            canvas,
            &text,
            Vec2::new(row.x + 48.0, center_y - size.y / 2.0),
            color,
        );
        let right_x = row.x + row.w - 48.0;
        match i {
            menus::OPTION_MODE
            | menus::OPTION_THEME
            | menus::OPTION_BACKGROUND
            | menus::OPTION_BLOCKS
            | menus::OPTION_BOARD => {
                let locked = i == menus::OPTION_MODE && mode_locked;
                let value = match i {
                    menus::OPTION_MODE => mode.name(),
                    menus::OPTION_THEME => theme,
                    menus::OPTION_BACKGROUND => background,
                    menus::OPTION_BLOCKS => style.block.name(),
                    _ => style.board.name(),
                };
                let center_x = right_x - slider_width / 2.0;
                draw_text_centered(
                    ctx,
                    canvas,
                    &ui_text(value, OPTION_LABEL_SIZE * 0.75),
                    Vec2::new(center_x, center_y),
                    if locked { LABEL_COLOR } else { color },
                )?;
                if locked {
                    continue;
                }
                // arrow blocks either side of the value
                let arrow = SLIDER_SEGMENT * 0.6;
                let arrow_color = if selected {
                    ACCENT_COLOR
                } else {
                    SLIDER_EMPTY_COLOR
                };
                for x in [right_x - slider_width, right_x - arrow] {
                    let rect = Rect::new(x, center_y - arrow / 2.0, arrow, arrow);
                    draw_block(canvas, meshes, rect, arrow_color, style.block);
                }
            }
            menus::OPTION_MUSIC => draw_slider(
                canvas,
                meshes,
                right_x,
                center_y,
                music_level,
                max_level,
                style.block,
            ),
            _ => draw_slider(
                canvas,
                meshes,
                right_x,
                center_y,
                sound_level,
                max_level,
                style.block,
            ),
        }
    }

    // controls reference, no hold in hard mode
    let rows: Vec<_> = CONTROLS
        .iter()
        .filter(|(action, _)| mode != Mode::Hard || *action != "Hold")
        .collect();
    let header = PANEL_PADDING + CONTROLS_ROW_HEIGHT * 1.2;
    let controls = Rect::new(
        panel.x,
        panel.y + panel.h + 12.0,
        panel.w,
        header + rows.len() as f32 * CONTROLS_ROW_HEIGHT + PANEL_PADDING,
    );
    draw_panel(canvas, meshes, controls);
    let left = controls.x + PANEL_PADDING + 28.0;
    let right = controls.x + controls.w - PANEL_PADDING - 28.0;
    draw_text(
        canvas,
        "Controls",
        Vec2::new(left, controls.y + PANEL_PADDING),
        SMALL_TEXT_SIZE * 1.25,
        LABEL_COLOR,
    );
    for (i, (action, keys)) in rows.into_iter().enumerate() {
        let y = controls.y + header + i as f32 * CONTROLS_ROW_HEIGHT;
        draw_text(
            canvas,
            action,
            Vec2::new(left, y),
            SMALL_TEXT_SIZE,
            LABEL_COLOR,
        );
        let keys = ui_text(keys, SMALL_TEXT_SIZE);
        let width = keys.measure_ui(ctx)?.x;
        draw_shadowed(canvas, &keys, Vec2::new(right - width, y), Color::WHITE);
    }

    draw_hint(
        ctx,
        canvas,
        "Up/Down: select   Left/Right: adjust   Esc: back",
        view_rect,
    )
}

pub fn draw_gameover(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    game_over: &GameOver,
    score: usize,
    level: usize,
) -> GameResult {
    draw_overlay_alpha(canvas, meshes, view_rect, game_over.overlay());
    let drop = game_over.panel();
    if drop <= 0.0 {
        return Ok(());
    }
    let time = game_over.time();
    let height = GAME_OVER_TITLE_ROW + 2.0 * GAME_OVER_STAT_ROW + PANEL_PADDING;
    let top = (view_rect.h - height) / 2.0 - (1.0 - ease_out_back(drop)) * view_rect.h;
    let panel = Rect::new(
        (view_rect.w - MENU_PANEL_WIDTH) / 2.0,
        top,
        MENU_PANEL_WIDTH,
        height,
    );
    draw_panel(canvas, meshes, panel);

    let title_center = Vec2::new(view_rect.w / 2.0, top + GAME_OVER_TITLE_ROW / 2.0 + 4.0);
    draw_dropping_title(ctx, canvas, game_over, "Game Over!", title_center)?;

    // final score and level count up
    let count = 1.0 - (1.0 - game_over.count()).powi(3);
    let stats = [
        ("Score", (score as f32 * count).round() as usize),
        ("Level", (level as f32 * count).round() as usize),
    ];
    let left = panel.x + 2.0 * PANEL_PADDING;
    let right = panel.x + panel.w - 2.0 * PANEL_PADDING;
    for (i, (label, value)) in stats.into_iter().enumerate() {
        let center_y = top + GAME_OVER_TITLE_ROW + (i as f32 + 0.5) * GAME_OVER_STAT_ROW;
        let label = ui_text(label, GAME_OVER_STAT_SIZE);
        let label_h = label.measure_ui(ctx)?.y;
        draw_shadowed(
            canvas,
            &label,
            Vec2::new(left, center_y - label_h / 2.0),
            LABEL_COLOR,
        );
        let value = ui_text(&value.to_string(), GAME_OVER_STAT_SIZE);
        let dims = value.measure_ui(ctx)?;
        let counting = game_over.count() > 0.0 && game_over.count() < 1.0;
        let color = if counting { ACCENT_COLOR } else { Color::WHITE };
        draw_shadowed(
            canvas,
            &value,
            Vec2::new(right - dims.x, center_y - dims.y / 2.0),
            color,
        );
    }

    if game_over.ready() {
        let blink = 0.65 + 0.35 * (time * 4.0).sin();
        let hint = ui_text("Press any key", SMALL_TEXT_SIZE);
        let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
        draw_text_centered(ctx, canvas, &hint, center, with_alpha(LABEL_COLOR, blink))?;
    }
    Ok(())
}

/// initials entry over the finished game
#[allow(clippy::too_many_arguments)]
pub fn draw_name_entry(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    initials: &Initials,
    score: usize,
    style: BlockStyle,
) -> GameResult {
    draw_overlay(canvas, meshes, view_rect);
    let height = 90.0 + 40.0 + NAME_CELL.y + 2.0 * NAME_ARROW + 3.0 * PANEL_PADDING;
    let panel = Rect::new(
        (view_rect.w - MENU_PANEL_WIDTH) / 2.0,
        (view_rect.h - height) / 2.0,
        MENU_PANEL_WIDTH,
        height,
    );
    draw_panel(canvas, meshes, panel);
    let time = ctx.time.time_since_start().as_secs_f32();
    draw_sparkles(canvas, meshes, panel, time);

    // title shrunk to fit the panel
    let title = "New High Score!";
    let width = piece_colored_text(title, NAME_TITLE_SIZE)
        .0
        .measure_ui(ctx)?
        .x;
    let size = NAME_TITLE_SIZE * ((panel.w - 2.0 * PANEL_PADDING) / width).min(1.0);
    let dims = piece_colored_text(title, size).0.measure_ui(ctx)?;
    let wobble = util::slow_wobble(time) * 3.0;
    let title_center = Vec2::new(view_rect.w / 2.0, panel.y + PANEL_PADDING + 30.0 + wobble);
    draw_piece_colored(
        canvas,
        title,
        title_center - Vec2::new(dims.x / 2.0, dims.y / 2.0),
        size,
    );
    draw_text_centered(
        ctx,
        canvas,
        &ui_text(&score.to_string(), UI_FONT_SIZE * 1.2),
        Vec2::new(view_rect.w / 2.0, panel.y + PANEL_PADDING + 90.0),
        ACCENT_COLOR,
    )?;

    // letter cells, the selected one blinks with arrows above and below
    let row_width = MAX_INITIALS as f32 * NAME_CELL.x + (MAX_INITIALS - 1) as f32 * NAME_CELL_GAP;
    let top = panel.y + 2.0 * PANEL_PADDING + 110.0 + NAME_ARROW;
    let bob = util::fast_wobble(time) * 2.0;
    for i in 0..MAX_INITIALS {
        let x = (view_rect.w - row_width) / 2.0 + i as f32 * (NAME_CELL.x + NAME_CELL_GAP);
        let cell = Rect::new(x, top, NAME_CELL.x, NAME_CELL.y);
        let selected = i == initials.cursor;
        let (fill, color) = if selected {
            let blink = 0.6 + 0.4 * (time * 6.0).sin();
            (
                with_alpha(HIGHLIGHT_COLOR, HIGHLIGHT_COLOR.a * (1.0 + blink)),
                mix_color(Color::WHITE, ACCENT_COLOR, blink),
            )
        } else {
            (SLIDER_EMPTY_COLOR, Color::WHITE)
        };
        draw_rect(canvas, meshes, cell, fill);
        if let Some(&letter) = initials.letters.get(i) {
            let text = ui_text(&(letter as char).to_string(), NAME_LETTER_SIZE);
            draw_text_centered(ctx, canvas, &text, cell.center().into(), color)?;
        }
        if selected {
            let arrow_x = cell.x + (cell.w - NAME_ARROW) / 2.0;
            for y in [cell.y - NAME_ARROW - 6.0 - bob, cell.y + cell.h + 6.0 + bob] {
                let rect = Rect::new(arrow_x, y, NAME_ARROW, NAME_ARROW);
                draw_block(canvas, meshes, rect, ACCENT_COLOR, style);
            }
        }
    }
    draw_hint(
        ctx,
        canvas,
        "Up/Down: letter   Left/Right: move   Backspace: delete   Enter: done",
        view_rect,
    )
}

/// top scores, with the new entry highlighted
pub fn draw_high_scores(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_rect: &Rect,
    scores: &ScoreBoard,
    highlight: Option<usize>,
    style: BlockStyle,
) -> GameResult {
    let panel = menu_panel_rect(
        view_rect,
        MAX_SCORES + 1,
        view_rect.h * 0.2,
        SCORES_PANEL_WIDTH,
        SCORES_ROW_HEIGHT,
    );
    draw_title(
        ctx,
        canvas,
        "High Scores",
        panel.y - PAUSED_TITLE_SIZE * 0.75,
        PAUSED_TITLE_SIZE,
        view_rect,
    )?;
    draw_panel(canvas, meshes, panel);

    // right aligned cells
    let draw_row =
        |canvas: &mut Canvas, row: Rect, cells: [&str; 5], size: f32, color: Color| -> GameResult {
            for (cell, right) in cells.iter().zip(SCORES_COLUMNS) {
                let text = ui_text(cell, size);
                let dims = text.measure_ui(ctx)?;
                let pos = Vec2::new(panel.x + right - dims.x, row.y + (row.h - dims.y) / 2.0);
                draw_shadowed(canvas, &text, pos, color);
            }
            Ok(())
        };
    let header = menu_row_rect(panel, 0, SCORES_ROW_HEIGHT);
    draw_row(
        canvas,
        header,
        ["#", "Name", "Score", "Level", "Lines"],
        SMALL_TEXT_SIZE,
        LABEL_COLOR,
    )?;

    let pulse = 0.5 + 0.5 * (ctx.time.time_since_start().as_secs_f32() * 5.0).sin();
    for i in 0..MAX_SCORES {
        let row = menu_row_rect(panel, i + 1, SCORES_ROW_HEIGHT);
        let rank = format!("{}.", i + 1);
        let Some(entry) = scores.scores.get(i) else {
            draw_row(
                canvas,
                row,
                [&rank, "---", "---", "---", "---"],
                SCORES_TEXT_SIZE,
                EMPTY_ROW_COLOR,
            )?;
            continue;
        };
        let color = if highlight == Some(i) {
            draw_selection(ctx, canvas, meshes, row, style);
            mix_color(ACCENT_COLOR, Color::WHITE, pulse)
        } else {
            Color::WHITE
        };
        draw_row(
            canvas,
            row,
            [
                &rank,
                &entry.initials,
                &entry.score.to_string(),
                &entry.level.to_string(),
                &entry.lines.to_string(),
            ],
            SCORES_TEXT_SIZE,
            color,
        )?;
    }
    draw_hint(ctx, canvas, "Press any key", view_rect)
}

/// piece colored letters that drop in one by one and bounce, then wave
fn draw_dropping_title(
    ctx: &Context,
    canvas: &mut Canvas,
    game_over: &GameOver,
    title: &str,
    center: Vec2,
) -> GameResult {
    let size = GAME_OVER_TITLE_SIZE;
    let dims = ui_text(title, size).measure_ui(ctx)?;
    let origin = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let shadow_offset = Vec2::splat(size / 16.0);
    let param = text_param();
    let time = game_over.time();
    for (i, (start, letter)) in title.char_indices().enumerate() {
        let drop = game_over.letter(i);
        if drop <= 0.0 || letter == ' ' {
            continue;
        }
        let x = ui_text(&title[..start], size).measure_ui(ctx)?.x;
        let wave = (time * 4.0 - i as f32 * 0.6).sin() * LETTER_WAVE * drop;
        let y = -(1.0 - ease_out_bounce(drop)) * LETTER_DROP + wave;
        let pos = origin + Vec2::new(x, y);
        let alpha = (drop * 4.0).min(1.0);
        let text = ui_text(&letter.to_string(), size);
        let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * alpha);
        canvas.draw(&text, param.dest(pos + shadow_offset).color(shadow));
        let color = tint(colors[i % colors.len()], TITLE_TINT);
        canvas.draw(
            &text,
            param.dest(pos).color(premultiply(with_alpha(color, alpha))),
        );
    }
    Ok(())
}

/// twinkling plus shapes along the panel edge
fn draw_sparkles(canvas: &mut Canvas, meshes: &Meshes, panel: Rect, time: f32) {
    for i in 0..SPARKLE_COUNT {
        let along = hash(i, 1.0);
        let pos = if i % 2 == 0 {
            Vec2::new(panel.x + along * panel.w, panel.y)
        } else {
            Vec2::new(panel.x + along * panel.w, panel.y + panel.h)
        };
        let phase = hash(i, 2.0) * std::f32::consts::TAU;
        let twinkle = ((time * 3.0 + phase).sin().max(0.0)).powi(3);
        if twinkle <= 0.01 {
            continue;
        }
        let s = SPARKLE_SIZE * (0.5 + twinkle);
        let color = with_alpha(LINE_CLEAR_COLOR, twinkle);
        draw_rect(
            canvas,
            meshes,
            Rect::new(pos.x - s / 2.0, pos.y - s / 6.0, s, s / 3.0),
            color,
        );
        draw_rect(
            canvas,
            meshes,
            Rect::new(pos.x - s / 6.0, pos.y - s / 2.0, s / 3.0, s),
            color,
        );
    }
}

fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}

fn ease_out_bounce(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}

/// particles, playfield flash and line clear callout
pub fn draw_effects(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    effects: &Effects,
    view_settings: &ViewSettings,
) -> GameResult {
    let playfield = view_settings.playfield_rect;
    let flash = effects.flash();
    if flash > 0.0 {
        draw_rect(canvas, meshes, playfield, with_alpha(Color::WHITE, flash));
    }
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let left = view_settings.staging_rect.x + 1.0;
    let bottom = playfield.y + playfield.h - 1.0;
    for trail in &effects.trails {
        let fade = trail.fade();
        let x = left + trail.x * cell;
        let length = trail.from - trail.to;
        for i in 0..TRAIL_SEGMENTS {
            let t = i as f32 / TRAIL_SEGMENTS as f32;
            let y0 = trail.to + length * t;
            let h = length / TRAIL_SEGMENTS as f32 * cell;
            let rect = snap_edges(Rect::new(x, bottom - y0 * cell - h, cell, h));
            let alpha = TRAIL_ALPHA * trail.strength * fade * (1.0 - t);
            draw_rect(canvas, meshes, rect, with_alpha(trail.color, alpha));
        }
    }
    if let Some(impact) = &effects.impact {
        let color = with_alpha(Color::WHITE, IMPACT_ALPHA * impact.strength * impact.fade());
        for block in impact.blocks {
            let rect =
                playfield_block_rect([block.x, block.y], &view_settings.staging_rect, &playfield);
            draw_rect(canvas, meshes, rect, color);
        }
    }
    for particle in effects.particles.iter().filter(|p| p.visible()) {
        let fade = particle.fade();
        let size = BLOCK_SIZE * particle.size * (0.4 + 0.6 * fade);
        let center = Vec2::new(left + particle.pos.x * cell, bottom - particle.pos.y * cell);
        let rect = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
        draw_rect(
            canvas,
            meshes,
            snap_rect(rect),
            with_alpha(particle.color, fade),
        );
    }

    if let Some(callout) = &effects.callout {
        let t = callout.progress();
        let quad = callout.lines >= 4;
        let base = if quad {
            CALLOUT_QUAD_SIZE
        } else {
            CALLOUT_SIZE
        };
        let mut size = base * (1.0 + CALLOUT_POP_SCALE * callout.pop());
        let width = ui_text(callout.text(), size).measure_ui(ctx)?.x;
        size *= ((playfield.w - 2.0 * PANEL_PADDING) / width).min(1.0);
        let alpha = ((1.0 - t) / 0.4).min(1.0);
        let center = Vec2::new(
            playfield.x + playfield.w / 2.0,
            bottom - (callout.row + t) * cell,
        );
        if quad {
            let (text, shadow) = piece_colored_text_alpha(callout.text(), size, alpha);
            let dims = text.measure_ui(ctx)?;
            let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
            let param = text_param();
            canvas.draw(&shadow, param.dest(pos + Vec2::splat(3.0)));
            canvas.draw(&text, param.dest(pos));
        } else {
            let color = with_alpha(LINE_CLEAR_COLOR, alpha);
            draw_text_centered(ctx, canvas, &ui_text(callout.text(), size), center, color)?;
        }
    }
    Ok(())
}

fn playfield_block_rect(block: [i32; 2], staging_rect: &Rect, playfield_rect: &Rect) -> Rect {
    // block[x,y] absolute units
    let x = staging_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = playfield_rect.y + playfield_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING))
        - 1.0;

    Rect::new(snap(x), snap(y), block_size(), block_size())
}

const DEBUG_ROW_HEIGHT: f32 = 26.0;
const DEBUG_PANEL_GAP: f32 = 24.0; // below the hold and next previews
const DEBUG_HINTS: [&str; 11] = [
    "Up/Down: pick",
    "Enter: restart",
    "R: replay on/off",
    "Left/Right: speed",
    "-/+: level",
    "Shift -/+: level x10",
    "P: pause",
    "B/S: styles",
    "H: hide panels",
    "F/Shift F: fps",
    "Esc: back",
];

/// animation viewer list and controls
#[allow(clippy::too_many_arguments)]
pub fn draw_debug(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_settings: &ViewSettings,
    names: &[&str],
    selected: usize,
    speed: f64,
    paused: bool,
    replay: bool,
    level: usize,
    fps: u32,
    style: BlockStyle,
) -> GameResult {
    let view = &view_settings.view_rect;
    let top = view_settings.preview_rect.bottom() + DEBUG_PANEL_GAP;

    // animations, right of the playfield
    let left = view_settings.preview_rect.x;
    let width = (view.w - left - PANEL_PADDING).max(view_settings.preview_rect.w);
    let rows = names.len() + 4;
    let panel = Rect::new(
        left,
        top,
        width,
        rows as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
    );
    draw_panel(canvas, meshes, panel);
    let title = ui_text("Animations", SMALL_TEXT_SIZE);
    let title_row = menu_row_rect(panel, 0, DEBUG_ROW_HEIGHT);
    draw_text_centered(ctx, canvas, &title, title_row.center().into(), LABEL_COLOR)?;
    for (i, name) in names.iter().enumerate() {
        let row = menu_row_rect(panel, i + 1, DEBUG_ROW_HEIGHT);
        let chosen = i == selected;
        if chosen {
            draw_selection(ctx, canvas, meshes, row, style);
        }
        let color = if chosen { ACCENT_COLOR } else { Color::WHITE };
        let text = ui_text(name, SMALL_TEXT_SIZE);
        draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
    }
    let status = if paused {
        "Paused".to_string()
    } else {
        format!("Speed {speed}x")
    };
    let replay = if replay { "Replay On" } else { "Replay Off" };
    let level = format!("Level {level}  {fps} FPS");
    for (i, line) in [status.as_str(), replay, level.as_str()]
        .into_iter()
        .enumerate()
    {
        let row = menu_row_rect(panel, rows - 3 + i, DEBUG_ROW_HEIGHT);
        let text = ui_text(line, SMALL_TEXT_SIZE);
        draw_text_centered(ctx, canvas, &text, row.center().into(), LABEL_COLOR)?;
    }

    // controls, left of the playfield
    let right = view_settings.hold_rect.right();
    let width = (right - PANEL_PADDING).max(view_settings.hold_rect.w);
    let panel = Rect::new(
        right - width,
        top,
        width,
        DEBUG_HINTS.len() as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
    );
    draw_panel(canvas, meshes, panel);
    for (i, hint) in DEBUG_HINTS.iter().enumerate() {
        let row = menu_row_rect(panel, i, DEBUG_ROW_HEIGHT);
        let text = ui_text(hint, SMALL_TEXT_SIZE * 0.75);
        draw_text_centered(ctx, canvas, &text, row.center().into(), LABEL_COLOR)?;
    }
    Ok(())
}
