use crate::gfx::{Color, Rect};
use crate::GameResult;

use crate::menus::Menu;
use crate::style::BlockStyle;
use crate::util;

use super::*;

pub(super) const PANEL_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
pub(super) const SELECTION_WORD_GAP: f32 = 24.0; // px from a bobbing block to a centered word, as from a row's end block to its label
pub(super) const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);

/// dims everything drawn before it
pub struct Overlay;

impl Layer for Overlay {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        draw_overlay(canvas, frame.meshes, &frame.view.view_rect);
        Ok(())
    }
}

pub(super) fn draw_overlay(canvas: &mut Painter, meshes: &Meshes, view_rect: &Rect) {
    draw_overlay_alpha(canvas, meshes, view_rect, 1.0);
}

pub(super) fn draw_overlay_alpha(
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
    alpha: f32,
) {
    let color = with_alpha(PAUSED_OVERLAY_COLOR, PAUSED_OVERLAY_COLOR.a * alpha);
    draw_rect(canvas, meshes, *view_rect, color);
}

pub(super) fn draw_panel(canvas: &mut Painter, meshes: &Meshes, rect: Rect) {
    draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    draw_rect(canvas, meshes, rect, PANEL_COLOR);
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
pub(super) fn draw_selection(
    time: f32,
    canvas: &mut Painter,
    meshes: &Meshes,
    row: Rect,
    style: BlockStyle,
) {
    draw_selection_around(time, canvas, meshes, row, None, style);
}

/// selection bar, with the bobbing blocks at its ends or hugging a centered word
pub(super) fn draw_selection_around(
    time: f32,
    canvas: &mut Painter,
    meshes: &Meshes,
    row: Rect,
    word_width: Option<f32>,
    style: BlockStyle,
) {
    draw_rect(canvas, meshes, row, HIGHLIGHT_COLOR);
    let bob = util::fast_wobble(time) * 3.0;
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
        draw_block(
            canvas,
            meshes,
            Rect::new(x, y, size, size),
            ACCENT_COLOR,
            style,
        );
    }
}

pub(super) fn draw_menu_list<T: Menu>(
    time: f32,
    canvas: &mut Painter,
    meshes: &Meshes,
    menu: &T,
    panel: Rect,
    style: BlockStyle,
) -> GameResult {
    for (i, item) in menu.items().iter().enumerate() {
        let row = menu_row_rect(panel, i, MENU_ROW_HEIGHT);
        let selected = menu.selected() == i;
        let mut text = item.clone();
        text.set_scale(px(MENU_ITEM_SIZE));
        if selected {
            let width = text.measure_ui().x;
            draw_selection_around(time, canvas, meshes, row, Some(width), style);
        }
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        draw_text_centered(canvas, &text, row.center(), color)?;
    }
    Ok(())
}

/// row of blocks filled up to the level, right aligned to right_x
pub(super) fn draw_slider(
    canvas: &mut Painter,
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
