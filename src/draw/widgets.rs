use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::util;

use super::*;

pub(super) const PANEL_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
pub(super) const SELECTION_WORD_GAP: f32 = 24.0; // px from a bobbing block to a centered word, as from a row's end block to its label
pub(super) const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);

/// dims everything drawn before it
pub struct Overlay;

impl Layer for Overlay {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        draw_overlay(canvas, 1.0);
        Ok(())
    }
}

pub(super) fn draw_overlay(canvas: &mut Painter, alpha: f32) {
    let color = with_alpha(PAUSED_OVERLAY_COLOR, PAUSED_OVERLAY_COLOR.a * alpha);
    draw_rect(canvas, canvas.view.view_rect, color);
}

pub(super) fn draw_panel(canvas: &mut Painter, rect: Rect) {
    draw_rect(canvas, frame_rect(rect), FRAME_COLOR);
    draw_rect(canvas, rect, PANEL_COLOR);
}

pub(super) fn menu_panel_rect(
    view_rect: &Rect,
    rows: usize,
    top: f32,
    width: f32,
    row_height: f32,
) -> Rect {
    let height = rows as f32 * row_height + 2.0 * PANEL_PADDING;
    Rect::new(view_rect.w / 2.0 - width / 2.0, top, width, height)
}

pub(super) fn menu_row_rect(panel: Rect, row: usize, row_height: f32) -> Rect {
    Rect::new(
        panel.x + PANEL_PADDING,
        panel.y + PANEL_PADDING + row as f32 * row_height,
        panel.w - 2.0 * PANEL_PADDING,
        row_height,
    )
}

/// highlight bar with a bobbing gold block at each end
pub(super) fn draw_selection(canvas: &mut Painter, row: Rect) {
    draw_selection_around(canvas, row, None);
}

/// selection bar, with the bobbing blocks at its ends or hugging a centered word
pub(super) fn draw_selection_around(canvas: &mut Painter, row: Rect, word_width: Option<f32>) {
    draw_rect(canvas, row, HIGHLIGHT_COLOR);
    let bob = util::fast_wobble(canvas.time()) * 3.0;
    let size = row.h * 0.3;
    let y = row.y + row.h / 2.0 - size / 2.0;
    let (left, right) = match word_width {
        // same gap to the word as a row's end block to its label
        Some(width) => {
            let center = row.x + row.w / 2.0;
            (
                center - width / 2.0 - SELECTION_WORD_GAP - size,
                center + width / 2.0 + SELECTION_WORD_GAP,
            )
        }
        None => (row.x + 12.0, row.x + row.w - 12.0 - size),
    };
    for x in [left + bob, right - bob] {
        draw_block(canvas, Rect::new(x, y, size, size), ACCENT_COLOR);
    }
}

/// row of blocks filled up to the level, right aligned to right_x
pub(super) fn draw_slider(canvas: &mut Painter, right_x: f32, center_y: f32, level: u8, max: u8) {
    let (segment, pitch) = (snap(SLIDER_SEGMENT), snap(SLIDER_SEGMENT + SLIDER_GAP));
    let left = snap(right_x - max as f32 * pitch + pitch - segment);
    let top = snap(center_y - segment / 2.0);
    for i in 0..max {
        let rect = Rect::new(left + i as f32 * pitch, top, segment, segment);
        if i < level {
            draw_block(canvas, rect, ACCENT_COLOR);
        } else {
            draw_rect(canvas, rect, SLIDER_EMPTY_COLOR);
        }
    }
}
