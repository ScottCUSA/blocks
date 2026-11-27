use ggez::glam::Vec2;
use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::scores::{Initials, MAX_INITIALS};
use crate::util;

use super::*;

pub(super) const SPARKLE_COUNT: usize = 14;
pub(super) const SPARKLE_SIZE: f32 = 8.0;
pub(super) const NAME_TITLE_SIZE: f32 = 40.0;
pub(super) const NAME_CELL: Vec2 = Vec2::new(72.0, 84.0);
pub(super) const NAME_CELL_GAP: f32 = 24.0;
pub(super) const NAME_LETTER_SIZE: f32 = 56.0;
pub(super) const NAME_ARROW: f32 = 14.0;

/// initials entry for a high score, over the game
pub struct NameEntryScreen<'a> {
    pub initials: &'a Initials,
    pub score: usize,
}

impl Layer for NameEntryScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let Self { initials, score } = *self;
        let view_rect = canvas.view.view_rect;
        draw_overlay(canvas, 1.0);
        let height = 90.0 + 40.0 + NAME_CELL.y + 2.0 * NAME_ARROW + 3.0 * PANEL_PADDING;
        let panel = centered_panel(&view_rect, height);
        draw_panel(canvas, panel);
        let time = canvas.time();
        draw_sparkles(canvas, panel, time);

        // title shrunk to fit the panel
        let title = "New High Score!";
        let width = piece_colored_text(title, NAME_TITLE_SIZE, 1.0)
            .0
            .measure_ui(canvas)?
            .x;
        let size = NAME_TITLE_SIZE * ((panel.w - 2.0 * PANEL_PADDING) / width).min(1.0);
        let dims = piece_colored_text(title, size, 1.0).0.measure_ui(canvas)?;
        let wobble = util::slow_wobble(time) * 3.0;
        let center_x = view_rect.w / 2.0;
        let title_center = Vec2::new(center_x, panel.y + PANEL_PADDING + 30.0 + wobble);
        draw_piece_colored(canvas, title, title_center - dims / 2.0, size);
        let score = ui_text(&score.to_string(), UI_FONT_SIZE * 1.2);
        let score_center = Vec2::new(center_x, panel.y + PANEL_PADDING + 90.0);
        draw_text_centered(canvas, &score, score_center, ACCENT_COLOR)?;

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
            draw_rect(canvas, cell, fill);
            if let Some(&letter) = initials.letters.get(i) {
                let text = ui_text(&(letter as char).to_string(), NAME_LETTER_SIZE);
                draw_text_centered(canvas, &text, cell.center().into(), color)?;
            }
            if selected {
                let arrow_x = cell.x + (cell.w - NAME_ARROW) / 2.0;
                for y in [cell.y - NAME_ARROW - 6.0 - bob, cell.y + cell.h + 6.0 + bob] {
                    let rect = Rect::new(arrow_x, y, NAME_ARROW, NAME_ARROW);
                    draw_block(canvas, rect, ACCENT_COLOR);
                }
            }
        }
        draw_hint(
            canvas,
            "Up/Down: letter   Left/Right: move   Backspace: delete   Enter: done",
        )
    }
}

/// twinkling plus shapes along the panel edge
pub(super) fn draw_sparkles(canvas: &mut Painter, panel: Rect, time: f32) {
    for i in 0..SPARKLE_COUNT {
        let along = hash(i, 1.0);
        let edge = if i % 2 == 0 { panel.y } else { panel.bottom() };
        let pos = Vec2::new(panel.x + along * panel.w, edge);
        let phase = hash(i, 2.0) * std::f32::consts::TAU;
        let twinkle = ((time * 3.0 + phase).sin().max(0.0)).powi(3);
        if twinkle <= 0.01 {
            continue;
        }
        let s = SPARKLE_SIZE * (0.5 + twinkle);
        let color = with_alpha(LINE_CLEAR_COLOR, twinkle);
        draw_rect(
            canvas,
            Rect::new(pos.x - s / 2.0, pos.y - s / 6.0, s, s / 3.0),
            color,
        );
        draw_rect(
            canvas,
            Rect::new(pos.x - s / 6.0, pos.y - s / 2.0, s / 3.0, s),
            color,
        );
    }
}
