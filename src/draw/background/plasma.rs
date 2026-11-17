use crate::gfx::Vec2;
use crate::gfx::{Color, Rect};

use crate::background::Background;
use crate::rustomino::RustominoType;
use crate::util;
use strum::IntoEnumIterator;

use super::*;
use crate::draw::*;

pub(super) const FROST_COLOR: Color = Color::new(0.7, 0.85, 1.0, 1.0);
pub(super) const MELT_ORDER: f32 = 0.6; // of the freeze timing
pub(super) const MELT_FADE: f32 = 0.4;
pub(super) const CRYSTAL_SEEDS: usize = 6;
pub(super) const CRYSTAL_GROW: f32 = 0.06; // seconds per cell
pub(super) const CRYSTAL_JITTER: f32 = 0.25;
pub(super) const CRYSTAL_FLASH: f32 = 0.35;
pub(super) const GLINT_PERIOD: f32 = 5.0;
pub(super) const GLINT_CHANCE: f32 = 0.04;
pub(super) const GLINT_TIME: f32 = 0.5;
pub(super) const PLASMA_CELL: f32 = 32.0; // px
pub(super) const RING_WIDTH: f32 = 70.0; // px, level up ring
pub(super) const PLASMA_SPEED: f32 = 0.8;
pub(super) const PLASMA_CALM_TWINKLE: f32 = 0.6; // twinkle depth when calm

/// level up ring strength at a point, 0.0 to 1.0
pub(super) fn level_ring(background: &Background, view_rect: &Rect, point: Vec2) -> (f32, Vec2) {
    let Some(p) = background.wave() else {
        return (0.0, Vec2::ZERO);
    };
    let center = Vec2::new(
        view_rect.x + view_rect.w / 2.0,
        view_rect.y + view_rect.h / 2.0,
    );
    let reach = center.distance(view_rect.point()) + RING_WIDTH * 2.0;
    let radius = (1.0 - (1.0 - p).powi(2)) * reach;
    let offset = point - center;
    let x = (offset.length() - radius) / RING_WIDTH;
    ((-x * x).exp() * (1.0 - p), offset.normalize_or_zero())
}

/// drain a color to icy gray
pub(super) fn frost(color: Color, amount: f32) -> Color {
    let ice = mix_color(util::rgb_to_grayscale(color), FROST_COLOR, 0.4);
    mix_color(color, with_alpha(ice, color.a), amount)
}

pub(super) struct Plasma;

impl Scenery for Plasma {
    const COLORS: (Color, Color) = PLASMA_COLORS;

    fn draw(
        &self,
        canvas: &mut Painter,
        meshes: &Meshes,
        view_rect: &Rect,
        background: &Background,
    ) {
        draw_plasma(canvas, meshes, view_rect, background);
    }
}

fn draw_plasma(canvas: &mut Painter, meshes: &Meshes, view_rect: &Rect, background: &Background) {
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
            // calm, still cells twinkle
            let calm = background.calm;
            let twinkle = (background.twinkle * (1.5 + 2.0 * hash(cell, 5.0))
                + hash(cell, 6.0) * std::f32::consts::TAU)
                .sin();
            let shimmer = 1.0 + calm * PLASMA_CALM_TWINKLE * twinkle;
            let alpha = ((0.01 + 0.04 * n * n) * glow * shimmer + 0.15 * wave).min(1.0);
            draw_rect(canvas, meshes, rect, with_alpha(color, alpha * melt));
        }
    }
}
