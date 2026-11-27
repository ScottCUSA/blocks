mod plasma;
mod stars;
mod storm;

pub use storm::{storm_strike_time, StrikeKind};

use plasma::*;
use stars::*;
use storm::*;

use ggez::graphics::{Color, Rect};

use crate::background::{Background, BackgroundStyle};

use crate::draw::*;

// top and bottom of the window per background
pub(super) const STARS_COLORS: (Color, Color) = (
    Color::new(0.01, 0.01, 0.04, 1.0),
    Color::new(0.06, 0.04, 0.16, 1.0),
);
pub(super) const PLASMA_COLORS: (Color, Color) = (
    Color::new(0.02, 0.01, 0.06, 1.0),
    Color::new(0.1, 0.04, 0.19, 1.0),
);
pub(super) const STORM_COLORS: (Color, Color) = (
    Color::new(0.02, 0.025, 0.045, 1.0),
    Color::new(0.08, 0.1, 0.14, 1.0),
);
pub(super) const OFF_COLORS: (Color, Color) = (
    Color::new(0.06, 0.07, 0.09, 1.0),
    Color::new(0.16, 0.18, 0.22, 1.0),
);

const CALM_DIM: f32 = 0.45; // darkening behind the title

fn colors(style: BackgroundStyle) -> (Color, Color) {
    match style {
        BackgroundStyle::Stars => STARS_COLORS,
        BackgroundStyle::Plasma => PLASMA_COLORS,
        BackgroundStyle::Storm => STORM_COLORS,
        BackgroundStyle::Off => OFF_COLORS,
    }
}

/// window clear color for a background
pub fn clear_color(style: BackgroundStyle) -> Color {
    colors(style).1
}

/// vertical gradient, then the animation
pub fn draw_background(canvas: &mut Painter, view_rect: &Rect, background: &Background) {
    let (top, bottom) = colors(background.style);
    draw_rect(canvas, *view_rect, bottom);
    draw_fade(canvas, *view_rect, top);
    match background.style {
        BackgroundStyle::Stars => draw_stars(canvas, view_rect, background),
        BackgroundStyle::Plasma => draw_plasma(canvas, view_rect, background),
        BackgroundStyle::Storm => draw_storm(canvas, view_rect, background),
        BackgroundStyle::Off => {}
    }
    if background.calm > 0.0 {
        let dim = Color::new(0.0, 0.0, 0.0, CALM_DIM * background.calm);
        draw_rect(canvas, *view_rect, dim);
    }
}

pub(super) fn smoothstep(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
