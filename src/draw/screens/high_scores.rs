use ggez::glam::Vec2;
use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::scores::{ScoreBoard, MAX_SCORES};

use super::*;

pub(super) const SCORES_PANEL_WIDTH: f32 = 680.0;
pub(super) const SCORES_ROW_HEIGHT: f32 = 40.0;
pub(super) const SCORES_TEXT_SIZE: f32 = 24.0;
// right edges of the rank, initials, score, level and lines columns
pub(super) const SCORES_COLUMNS: [f32; 5] = [90.0, 210.0, 430.0, 530.0, 630.0];
pub(super) const EMPTY_ROW_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.35);

/// top scores, with the new entry highlighted
pub struct HighScoresScreen<'a> {
    pub scores: &'a ScoreBoard,
    pub highlight: Option<usize>,
}

impl Layer for HighScoresScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let Self { scores, highlight } = *self;
        let view_rect = canvas.view.view_rect;
        let (top, rows) = (view_rect.h * 0.2, MAX_SCORES + 1);
        let panel = menu_panel_rect(&view_rect, rows, top, SCORES_PANEL_WIDTH, SCORES_ROW_HEIGHT);
        draw_title(
            canvas,
            "High Scores",
            top - PAUSED_TITLE_SIZE * 0.75,
            PAUSED_TITLE_SIZE,
        )?;
        draw_panel(canvas, panel);

        // right aligned cells
        let draw_row = |canvas: &mut Painter,
                        row: Rect,
                        cells: [&str; 5],
                        size: f32,
                        color: Color|
         -> GameResult {
            for (cell, right) in cells.iter().zip(SCORES_COLUMNS) {
                let text = ui_text(cell, size);
                let dims = text.measure_ui(canvas)?;
                let pos = Vec2::new(panel.x + right - dims.x, row.y + (row.h - dims.y) / 2.0);
                draw_shadowed(canvas, &text, pos, color);
            }
            Ok(())
        };
        let header = menu_row_rect(panel, 0, SCORES_ROW_HEIGHT);
        let titles = ["#", "Name", "Score", "Level", "Lines"];
        draw_row(canvas, header, titles, SMALL_TEXT_SIZE, LABEL_COLOR)?;

        let pulse = 0.5 + 0.5 * (canvas.time() * 5.0).sin();
        for i in 0..MAX_SCORES {
            let row = menu_row_rect(panel, i + 1, SCORES_ROW_HEIGHT);
            let rank = format!("{}.", i + 1);
            let Some(entry) = scores.scores.get(i) else {
                let empty = [rank.as_str(), "---", "---", "---", "---"];
                draw_row(canvas, row, empty, SCORES_TEXT_SIZE, EMPTY_ROW_COLOR)?;
                continue;
            };
            let color = if highlight == Some(i) {
                draw_selection(canvas, row);
                mix_color(ACCENT_COLOR, Color::WHITE, pulse)
            } else {
                Color::WHITE
            };
            let numbers = [entry.score, entry.level, entry.lines].map(|n| n.to_string());
            let [score, level, lines] = numbers.each_ref().map(String::as_str);
            let cells = [rank.as_str(), &entry.initials, score, level, lines];
            draw_row(canvas, row, cells, SCORES_TEXT_SIZE, color)?;
        }
        draw_hint(canvas, "Press any key")
    }
}
