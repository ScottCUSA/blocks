use crate::gfx::Vec2;
use crate::gfx::{Color, DrawParam, Rect};

use crate::style::BlockStyle;

use super::*;

pub(super) const NEON_EDGE: f32 = 3.0;
pub(super) const RETRO_EDGE: f32 = 2.0;
pub(super) const BEVEL: f32 = 3.0; // block edge highlight and shadow width

/// gap line filling the snapped gap between blocks
pub(super) fn snap_line(start: f32) -> (f32, f32) {
    let width = CELL - block_size();
    (snap(start + BLOCK_PADDING) - width, width)
}

pub(super) fn draw_rect(canvas: &mut Painter, meshes: &Meshes, rect: Rect, color: Color) {
    draw_mesh(
        canvas,
        &meshes.rect,
        DrawParam::default().dest_rect(rect).color(color),
    );
}

pub(super) fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

pub(super) fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * amount,
        from.g + (to.g - from.g) * amount,
        from.b + (to.b - from.b) * amount,
        from.a + (to.a - from.a) * amount,
    )
}

pub(super) fn shade(color: Color, amount: f32) -> Color {
    Color::new(
        color.r * amount,
        color.g * amount,
        color.b * amount,
        color.a,
    )
}

pub(super) fn tint(color: Color, amount: f32) -> Color {
    Color::new(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

pub(super) fn draw_rounded(canvas: &mut Painter, meshes: &Meshes, rect: Rect, color: Color) {
    draw_mesh(
        canvas,
        &meshes.rounded,
        DrawParam::default()
            .dest(Vec2::new(rect.x, rect.y))
            .scale(Vec2::new(rect.w / BLOCK_SIZE, rect.h / BLOCK_SIZE))
            .color(color),
    );
}

pub(super) fn inset(rect: Rect, by: f32) -> Rect {
    Rect::new(
        rect.x + by,
        rect.y + by,
        rect.w - 2.0 * by,
        rect.h - 2.0 * by,
    )
}

/// shape of a block layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Square,
    Rounded,        // jelly corners, scaled with the rect
    RoundedOutline, // jelly ghost cell
}

/// one rect of a block, bottom first
pub type BlockLayer = (Rect, Color, Shape);

/// layers of a block in the selected style, edges scale with the block size
pub(super) fn block_layers(rect: Rect, color: Color, style: BlockStyle) -> Vec<BlockLayer> {
    use Shape::*;
    let k = rect.w.min(rect.h) / BLOCK_SIZE;
    match style {
        BlockStyle::Bevel => {
            // light top left edge, dark bottom right edge and gloss
            let bevel = BEVEL * k;
            let highlight = Rect::new(rect.x, rect.y, rect.w - bevel, rect.h - bevel);
            let face = inset(rect, bevel);
            let gloss = Rect::new(face.x, face.y, face.w, face.h * 0.45);
            vec![
                (rect, shade(color, 0.55), Square),
                (highlight, tint(color, 0.45), Square),
                (face, color, Square),
                (gloss, with_alpha(Color::WHITE, 0.12 * color.a), Square),
            ]
        }
        BlockStyle::Flat => vec![
            (rect, shade(color, 0.8), Square),
            (inset(rect, 2.0 * k), color, Square),
        ],
        BlockStyle::Neon => {
            // dim core with a bright rim
            let mut layers = vec![(rect, with_alpha(shade(color, 0.3), color.a * 0.85), Square)];
            let rim = tint(color, 0.15);
            layers.extend(outline_rects(rect, NEON_EDGE * k).map(|edge| (edge, rim, Square)));
            let core = Rect::new(
                rect.x + rect.w * 0.36,
                rect.y + rect.h * 0.36,
                rect.w * 0.28,
                rect.h * 0.28,
            );
            layers.push((core, with_alpha(tint(color, 0.5), color.a * 0.7), Square));
            layers
        }
        BlockStyle::Retro => {
            // dark border, solid face and a pixel highlight in the corner
            let edge = RETRO_EDGE * k;
            let pixel = (rect.w.min(rect.h) / 7.5).round().max(1.0);
            let face = inset(rect, edge);
            let shine = with_alpha(tint(color, 0.85), color.a);
            let (x, y) = (face.x + pixel, face.y + pixel);
            vec![
                (rect, shade(color, 0.25), Square),
                (face, color, Square),
                (Rect::new(x, y, pixel, pixel), shine, Square),
                (Rect::new(x + pixel, y, pixel * 2.0, pixel), shine, Square),
                (Rect::new(x, y + pixel, pixel, pixel * 2.0), shine, Square),
            ]
        }
        BlockStyle::Jelly => {
            // rounded with a soft shadow and a glossy top
            let face = Rect::new(rect.x, rect.y, rect.w, rect.h - 2.0 * k);
            let gloss = Rect::new(
                face.x + face.w * 0.2,
                face.y + face.h * 0.12,
                face.w * 0.6,
                face.h * 0.3,
            );
            vec![
                (rect, shade(color, 0.6), Rounded),
                (face, color, Rounded),
                (gloss, with_alpha(Color::WHITE, 0.35 * color.a), Rounded),
            ]
        }
    }
}

/// rect edges of the given width, as non-overlapping rects
pub(super) fn outline_rects(rect: Rect, width: f32) -> [Rect; 4] {
    let (x, y, w, h) = (rect.x, rect.y, rect.w, rect.h);
    [
        Rect::new(x, y, w, width),
        Rect::new(x, y + h - width, w, width),
        Rect::new(x, y + width, width, h - 2.0 * width),
        Rect::new(x + w - width, y + width, width, h - 2.0 * width),
    ]
}

/// block in the selected style, painted
pub(super) fn draw_block(
    canvas: &mut Painter,
    meshes: &Meshes,
    rect: Rect,
    color: Color,
    style: BlockStyle,
) {
    for (rect, color, shape) in block_layers(rect, color, style) {
        match shape {
            Shape::Square => draw_rect(canvas, meshes, rect, color),
            Shape::Rounded => draw_rounded(canvas, meshes, rect, color),
            Shape::RoundedOutline => {}
        }
    }
}

pub(super) fn frame_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x - FRAME_WIDTH,
        rect.y - FRAME_WIDTH,
        rect.w + 2.0 * FRAME_WIDTH,
        rect.h + 2.0 * FRAME_WIDTH,
    )
}

/// to the nearest whole physical pixel
pub(super) fn snap(v: f32) -> f32 {
    let scale = pixel_scale();
    (v * scale).round() / scale
}

/// position and size snapped, so moving rects keep their size
pub(super) fn snap_rect(rect: Rect) -> Rect {
    Rect::new(snap(rect.x), snap(rect.y), snap(rect.w), snap(rect.h))
}

/// edges snapped, so adjacent rects still meet
pub(super) fn snap_edges(rect: Rect) -> Rect {
    let (x, y) = (snap(rect.x), snap(rect.y));
    Rect::new(x, y, snap(rect.x + rect.w) - x, snap(rect.y + rect.h) - y)
}

/// block size in whole pixels, with at least a pixel of gap
pub(super) fn block_size() -> f32 {
    let scale = pixel_scale();
    let cell = (CELL * scale).round();
    (BLOCK_SIZE * scale).round().min(cell - 1.0).max(1.0) / scale
}

/// stable pseudo random value in 0.0..1.0 for item i
pub(super) fn hash(i: usize, n: f32) -> f32 {
    ((i as f32 * 12.9898 + n * 78.233).sin() * 43758.547)
        .fract()
        .abs()
}
