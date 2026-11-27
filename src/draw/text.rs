use ggez::glam::Vec2;
use ggez::graphics::{self, Color, TextFragment};
use ggez::GameResult;

use crate::rustomino::RustominoType;
use crate::util;
use std::sync::atomic::{AtomicU32, Ordering};
use strum::IntoEnumIterator;

use super::*;

// physical pixels per view unit, text is rasterized at the physical size
pub(super) static PIXEL_SCALE: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub fn set_pixel_scale(scale: f32) {
    PIXEL_SCALE.store(scale.to_bits(), Ordering::Relaxed);
}

pub(super) fn pixel_scale() -> f32 {
    f32::from_bits(PIXEL_SCALE.load(Ordering::Relaxed))
}

pub(super) fn px(size: f32) -> graphics::PxScale {
    graphics::PxScale::from(size * pixel_scale())
}

pub(super) fn text_param() -> graphics::DrawParam {
    graphics::DrawParam::default().scale(Vec2::splat(1.0 / pixel_scale()))
}

pub(super) trait MeasureUi {
    /// size in view units
    fn measure_ui(&self, canvas: &Painter) -> GameResult<Vec2>;
}

impl MeasureUi for graphics::Text {
    fn measure_ui(&self, canvas: &Painter) -> GameResult<Vec2> {
        Ok(Vec2::from(self.measure(canvas.ctx)?) / pixel_scale())
    }
}

pub(super) fn ui_text(text: &str, size: f32) -> graphics::Text {
    let mut text = graphics::Text::new(text);
    text.set_font("04b30").set_scale(px(size));
    text
}

pub(super) fn draw_shadowed(canvas: &mut Painter, text: &graphics::Text, pos: Vec2, color: Color) {
    let param = text_param();
    // the shadow fades with the text
    let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * color.a);
    draw(
        canvas,
        text,
        param.dest(pos + Vec2::splat(2.0)).color(shadow),
    );
    draw(canvas, text, param.dest(pos).color(premultiply(color)));
}

/// text in the ui font with a drop shadow
pub(super) fn draw_text(canvas: &mut Painter, text: &str, pos: Vec2, size: f32, color: Color) {
    draw_shadowed(canvas, &ui_text(text, size), pos, color);
}

/// text positioned by its center
pub(super) fn draw_text_centered(
    canvas: &mut Painter,
    text: &graphics::Text,
    center: Vec2,
    color: Color,
) -> GameResult {
    let size = text.measure_ui(canvas)?;
    draw_shadowed(canvas, text, center - size / 2.0, color);
    Ok(())
}

/// text with each letter colored like a piece, and a matching shadow text
/// per letter colors override the draw color, so the shadow is its own text
pub(super) fn piece_colored_text(
    text: &str,
    size: f32,
    alpha: f32,
) -> (graphics::Text, graphics::Text) {
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

pub(super) fn draw_piece_colored(canvas: &mut Painter, text: &str, pos: Vec2, size: f32) {
    let (text, shadow) = piece_colored_text(text, size, 1.0);
    let offset = Vec2::splat((size / 16.0).max(2.0));
    draw(canvas, &shadow, text_param().dest(pos + offset));
    draw(canvas, &text, text_param().dest(pos));
}

/// large wobbling piece colored title
pub(super) fn draw_title(
    canvas: &mut Painter,
    title: &str,
    center_y: f32,
    size: f32,
) -> GameResult {
    let wobble = util::slow_wobble(canvas.time());
    let center = Vec2::new(
        canvas.view.view_rect.w / 2.0,
        center_y + wobble * size / 12.0,
    );
    let dims = piece_colored_text(title, size, 1.0).0.measure_ui(canvas)?;
    draw_piece_colored(canvas, title, center - dims / 2.0, size);
    Ok(())
}

pub(super) fn draw_hint(canvas: &mut Painter, hint: &str) -> GameResult {
    let view_rect = canvas.view.view_rect;
    let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
    draw_text_centered(canvas, &ui_text(hint, SMALL_TEXT_SIZE), center, LABEL_COLOR)
}
