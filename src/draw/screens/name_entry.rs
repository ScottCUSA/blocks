use crate::gfx::Vec2;
use crate::gfx::{Color, Rect};
use crate::GameResult;

use crate::scores::{Initials, MAX_INITIALS};
use crate::util;

use super::*;
use crate::draw::*;

pub(super) const SPARKLE_COUNT: usize = 14;
pub(super) const SPARKLE_SIZE: f32 = 8.0;
pub(super) const NAME_TITLE_SIZE: f32 = 40.0;
pub(super) const NAME_CELL: Vec2 = Vec2::new(72.0, 84.0);
pub(super) const NAME_CELL_GAP: f32 = 24.0;
pub(super) const NAME_LETTER_SIZE: f32 = 56.0;
pub(super) const NAME_ARROW: f32 = 14.0;

/// initials entry over the finished game
/// initials entry for a high score, over the game
pub struct NameEntryScreen<'a> {
    pub initials: &'a Initials,
    pub score: usize,
}

impl Layer for NameEntryScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self { initials, score } = *self;
        let time = frame.time;
        let meshes = frame.meshes;
        let view_rect = &frame.view.view_rect;
        let style = frame.block;
        draw_overlay(canvas, meshes, view_rect);
        let height = 90.0 + 40.0 + NAME_CELL.y + 2.0 * NAME_ARROW + 3.0 * PANEL_PADDING;
        let panel = Rect::new(
            (view_rect.w - MENU_PANEL_WIDTH) / 2.0,
            (view_rect.h - height) / 2.0,
            MENU_PANEL_WIDTH,
            height,
        );
        draw_panel(canvas, meshes, panel);
        draw_sparkles(canvas, meshes, panel, time);

        // title shrunk to fit the panel
        let title = "New High Score!";
        let width = piece_colored_text(title, NAME_TITLE_SIZE).0.measure_ui().x;
        let size = NAME_TITLE_SIZE * ((panel.w - 2.0 * PANEL_PADDING) / width).min(1.0);
        let dims = piece_colored_text(title, size).0.measure_ui();
        let wobble = util::slow_wobble(time) * 3.0;
        let title_center = Vec2::new(view_rect.w / 2.0, panel.y + PANEL_PADDING + 30.0 + wobble);
        draw_piece_colored(
            canvas,
            title,
            title_center - Vec2::new(dims.x / 2.0, dims.y / 2.0),
            size,
        );
        draw_text_centered(
            canvas,
            &ui_text(&score.to_string(), UI_FONT_SIZE * 1.2),
            Vec2::new(view_rect.w / 2.0, panel.y + PANEL_PADDING + 90.0),
            ACCENT_COLOR,
        )?;

        // letter cells, the selected one blinks with arrows above and below
        let row_width =
            MAX_INITIALS as f32 * NAME_CELL.x + (MAX_INITIALS - 1) as f32 * NAME_CELL_GAP;
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
                draw_text_centered(canvas, &text, cell.center(), color)?;
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
            canvas,
            "Up/Down: letter   Left/Right: move   Backspace: delete   Enter: done",
            view_rect,
        )
    }
}

/// twinkling plus shapes along the panel edge
pub(super) fn draw_sparkles(canvas: &mut Painter, meshes: &Meshes, panel: Rect, time: f32) {
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
