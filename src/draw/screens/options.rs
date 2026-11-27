use ggez::glam::Vec2;
use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::game::Mode;
use crate::menus;

use super::*;

pub(super) const OPTION_LABEL_SIZE: f32 = 32.0;
pub(super) const OPTIONS_PANEL_WIDTH: f32 = 680.0;
pub(super) const OPTION_ROW_HEIGHT: f32 = 40.0;
pub(super) const BACK_GAP: f32 = 7.0; // px above Back on the options pages
pub(super) const CONTROLS_ROW_HEIGHT: f32 = 22.0;
pub(super) const CONTROLS: [(&str, &str); 6] = [
    ("Move", "Left/Right  A/D"),
    ("Rotate", "Up/W  Ctrl/Z"),
    ("Soft drop", "Down/S"),
    ("Hard drop", "Space"),
    ("Hold", "Shift/C"),
    ("Pause", "Esc"),
];

/// what an options row shows on its right
pub enum OptionRow<'a> {
    Choice(&'a str, bool), // value, locked
    Slider(u8),
    Link, // opens another page: centered like Back, at the label size
    Back,
}

/// options and audio pages
pub struct OptionsScreen<'a> {
    pub title: &'a str,
    pub menu: &'a menus::MenuList,
    pub rows: &'a [OptionRow<'a>],
    pub mode: Mode,
    pub max_level: u8,
}

impl Layer for OptionsScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let Self {
            title,
            menu,
            rows,
            mode,
            max_level,
        } = *self;
        let view_rect = canvas.view.view_rect;
        draw_title(canvas, title, view_rect.h * 0.12, TITLE_SIZE)?;

        // settings, sliders and back, a little space above Back so it reads as its own button
        let (top, count) = (view_rect.h * 0.19, menu.items.len());
        let mut panel = menu_panel_rect(
            &view_rect,
            count,
            top,
            OPTIONS_PANEL_WIDTH,
            OPTION_ROW_HEIGHT,
        );
        panel.h += BACK_GAP;
        let slider_width = max_level as f32 * (SLIDER_SEGMENT + SLIDER_GAP) - SLIDER_GAP;
        draw_panel(canvas, panel);
        for (i, (item, kind)) in menu.items.iter().zip(rows).enumerate() {
            let mut row = menu_row_rect(panel, i, OPTION_ROW_HEIGHT);
            let selected = menu.selected() == i;
            let color = if selected { ACCENT_COLOR } else { Color::WHITE };
            // centered words, the blocks hug the word
            let centered = match kind {
                OptionRow::Back => Some(MENU_ITEM_SIZE),
                OptionRow::Link => Some(OPTION_LABEL_SIZE),
                _ => None,
            };
            if let Some(size) = centered {
                if let OptionRow::Back = kind {
                    row.y += BACK_GAP;
                }
                let text = ui_text(item, size);
                if selected {
                    let width = text.measure_ui(canvas)?.x;
                    draw_selection_around(canvas, row, Some(width));
                }
                draw_text_centered(canvas, &text, row.center().into(), color)?;
                continue;
            }
            if selected {
                draw_selection(canvas, row);
            }

            // label on the left, setting on the right
            let text = ui_text(item, OPTION_LABEL_SIZE);
            let center_y = row.y + row.h / 2.0;
            let label_y = center_y - text.measure_ui(canvas)?.y / 2.0;
            draw_shadowed(canvas, &text, Vec2::new(row.x + 48.0, label_y), color);
            let right_x = row.x + row.w - 48.0;
            let arrow = SLIDER_SEGMENT * 0.6;
            let arrow_color = if selected {
                ACCENT_COLOR
            } else {
                SLIDER_EMPTY_COLOR
            };
            match kind {
                OptionRow::Choice(value, locked) => {
                    let value = ui_text(value, OPTION_LABEL_SIZE * 0.75);
                    let center = Vec2::new(right_x - slider_width / 2.0, center_y);
                    let value_color = if *locked { LABEL_COLOR } else { color };
                    draw_text_centered(canvas, &value, center, value_color)?;
                    if *locked {
                        continue;
                    }
                    // arrow blocks either side of the value
                    for x in [right_x - slider_width, right_x - arrow] {
                        let rect = Rect::new(x, center_y - arrow / 2.0, arrow, arrow);
                        draw_block(canvas, rect, arrow_color);
                    }
                }
                OptionRow::Slider(level) => {
                    draw_slider(canvas, right_x, center_y, *level, max_level)
                }
                OptionRow::Back | OptionRow::Link => {}
            }
        }

        // controls reference, no hold in hard mode
        let rows: Vec<_> = CONTROLS
            .iter()
            .filter(|(action, _)| mode != Mode::Hard || *action != "Hold")
            .collect();
        let header = PANEL_PADDING + CONTROLS_ROW_HEIGHT * 1.2;
        let height = header + rows.len() as f32 * CONTROLS_ROW_HEIGHT + PANEL_PADDING;
        let controls = Rect::new(panel.x, panel.bottom() + 12.0, panel.w, height);
        draw_panel(canvas, controls);
        let left = controls.x + PANEL_PADDING + 28.0;
        let right = controls.right() - PANEL_PADDING - 28.0;
        let heading = Vec2::new(left, controls.y + PANEL_PADDING);
        draw_text(
            canvas,
            "Controls",
            heading,
            SMALL_TEXT_SIZE * 1.25,
            LABEL_COLOR,
        );
        for (i, (action, keys)) in rows.into_iter().enumerate() {
            let y = controls.y + header + i as f32 * CONTROLS_ROW_HEIGHT;
            draw_text(
                canvas,
                action,
                Vec2::new(left, y),
                SMALL_TEXT_SIZE,
                LABEL_COLOR,
            );
            let keys = ui_text(keys, SMALL_TEXT_SIZE);
            let width = keys.measure_ui(canvas)?.x;
            draw_shadowed(canvas, &keys, Vec2::new(right - width, y), Color::WHITE);
        }
        draw_hint(
            canvas,
            "Up/Down: select   Left/Right: adjust   Enter: open   Esc: back",
        )
    }
}
