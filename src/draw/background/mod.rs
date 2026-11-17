mod plasma;
mod stars;
mod storm;

pub use storm::{storm_strike_time, StrikeKind};

use plasma::*;
use stars::*;
use storm::*;

use ggez::graphics::{self, Color, Rect};

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

/// a background style's gradient and animation, statically dispatched
pub(super) trait Scenery {
    /// window gradient, top and bottom
    const COLORS: (Color, Color);

    /// animation over the gradient
    fn draw(
        &self,
        _canvas: &mut Painter,
        _meshes: &Meshes,
        _view_rect: &Rect,
        _background: &Background,
    ) {
    }
}

/// gradient only
struct Off;

impl Scenery for Off {
    const COLORS: (Color, Color) = OFF_COLORS;
}

fn colors(style: BackgroundStyle) -> (Color, Color) {
    match style {
        BackgroundStyle::Stars => Stars::COLORS,
        BackgroundStyle::Plasma => Plasma::COLORS,
        BackgroundStyle::Storm => Storm::COLORS,
        BackgroundStyle::Off => Off::COLORS,
    }
}

/// window clear color for a background
pub fn clear_color(style: BackgroundStyle) -> Color {
    colors(style).1
}

/// vertical gradient, then the animation
pub fn draw_background(
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    background: &Background,
) {
    match background.style {
        BackgroundStyle::Stars => paint(&Stars, canvas, meshes, view_rect, background),
        BackgroundStyle::Plasma => paint(&Plasma, canvas, meshes, view_rect, background),
        BackgroundStyle::Storm => paint(&Storm, canvas, meshes, view_rect, background),
        BackgroundStyle::Off => paint(&Off, canvas, meshes, view_rect, background),
    }
}

fn paint<S: Scenery>(
    scenery: &S,
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    background: &Background,
) {
    let (top, bottom) = S::COLORS;
    draw_rect(canvas, meshes, *view_rect, bottom);
    draw_mesh(
        canvas,
        &meshes.background,
        graphics::DrawParam::default()
            .dest_rect(*view_rect)
            .color(top),
    );
    scenery.draw(canvas, meshes, view_rect, background);
    if background.calm > 0.0 {
        draw_rect(
            canvas,
            meshes,
            *view_rect,
            Color::new(0.0, 0.0, 0.0, CALM_DIM * background.calm),
        );
    }
}

pub(super) fn smoothstep(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    // catches swapped match arms, no screenshot scene uses Off
    #[test]
    fn colors_match_style() {
        assert_eq!(colors(BackgroundStyle::Stars), STARS_COLORS);
        assert_eq!(colors(BackgroundStyle::Plasma), PLASMA_COLORS);
        assert_eq!(colors(BackgroundStyle::Storm), STORM_COLORS);
        assert_eq!(colors(BackgroundStyle::Off), OFF_COLORS);
    }

    #[test]
    fn clear_color_is_gradient_bottom() {
        for style in BackgroundStyle::ALL {
            assert_eq!(clear_color(style), colors(style).1);
        }
    }
}
