use ggez::glam::Vec2;
use ggez::graphics::{Color, Rect};

use crate::rustomino::{Rustomino, RustominoType};
use crate::style::BlockStyle;
use strum::IntoEnumIterator;

use super::*;
use crate::draw::*;

pub(super) const TITLE_COLORS: (Color, Color) = (
    Color::new(0.02, 0.03, 0.07, 1.0),
    Color::new(0.06, 0.08, 0.17, 1.0),
);
const TITLE_PIECES: usize = 36;
const TITLE_FALL: f32 = 0.06; // view heights per second, nearest
const TITLE_SPIN: f32 = 0.25; // quarter turns per second, fastest
const TITLE_BLOCK_NEAR: f32 = 1.1; // block size scale, nearest
const TITLE_BLOCK_FAR: f32 = 0.35;
const TITLE_STRENGTH: f32 = 0.45; // color over the gradient, nearest

/// pieces drifting down behind the title screens
pub fn draw_title_backdrop(
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    time: f32,
    style: BlockStyle,
) {
    let (top, bottom) = TITLE_COLORS;
    draw_rect(canvas, meshes, *view_rect, bottom);
    draw_mesh(
        canvas,
        &meshes.background,
        ggez::graphics::DrawParam::default()
            .dest_rect(*view_rect)
            .color(top),
    );
    let types: Vec<RustominoType> = RustominoType::iter().collect();
    // far first
    for i in 0..TITLE_PIECES {
        let n = |k: f32| hash(i, k + 900.0);
        let depth = (i as f32 + n(1.0)) / TITLE_PIECES as f32;
        let block = BLOCK_SIZE * (TITLE_BLOCK_FAR + (TITLE_BLOCK_NEAR - TITLE_BLOCK_FAR) * depth);
        let margin = block * 4.0;
        let span = view_rect.h + 2.0 * margin;
        let fall = TITLE_FALL * (0.3 + 0.7 * depth) * view_rect.h / span;
        let y = view_rect.y - margin + (n(2.0) + time * fall).fract() * span;
        let x = view_rect.x + n(3.0) * view_rect.w;
        let rtype = types[i % types.len()];
        let spin = TITLE_SPIN * (0.4 + 0.6 * n(4.0));
        let turns = (n(5.0) * 4.0) as i32 + (time * spin + n(6.0)) as i32;
        let blocks = Rustomino::new(rtype).blocks.map(|b| {
            let mut b = b.as_vec2();
            for _ in 0..turns.rem_euclid(4) {
                b = Vec2::new(-b.y, b.x);
            }
            b
        });
        let center = blocks.iter().copied().sum::<Vec2>() / 4.0;
        let strength = TITLE_STRENGTH * (0.25 + 0.75 * depth);
        let color = mix_color(mix_color(top, bottom, 0.5), rtype.color(), strength);
        for b in blocks {
            let at = Vec2::new(x, y) + (b - center - Vec2::splat(0.5)) * block;
            let rect = Rect::new(at.x, at.y, block, block);
            if overlaps(rect, view_rect) {
                draw_block(canvas, meshes, rect, color, style);
            }
        }
    }
}

/// window clear color behind the title screens
pub fn title_clear_color() -> Color {
    TITLE_COLORS.1
}
