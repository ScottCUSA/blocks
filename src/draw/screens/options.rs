use crate::gfx::Vec2;
use crate::gfx::{Color, Rect};
use crate::GameResult;

use crate::game::board::Mode;
use crate::menus::{self, Menu};
use crate::style::Style;

use super::*;
use crate::draw::*;

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
    pub style: &'a Style,
    pub max_level: u8,
}

impl Layer for OptionsScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self {
            title,
            menu,
            rows,
            mode,
            style,
            max_level,
        } = *self;
        let time = frame.time;
        let meshes = frame.meshes;
        let view_settings = frame.view;
        let view_rect = &view_settings.view_rect;
        draw_title(
            time,
            canvas,
            title,
            view_rect.h * 0.12,
            TITLE_SIZE,
            view_rect,
        )?;

        // settings, sliders and back
        let panel = menu_panel_rect(
            view_rect,
            menu.items().len(),
            view_rect.h * 0.19,
            OPTIONS_PANEL_WIDTH,
            OPTION_ROW_HEIGHT,
        );
        // a little space above Back so it reads as its own button
        let back_gap = BACK_GAP;
        let mut panel = panel;
        panel.h += back_gap;
        let slider_width = max_level as f32 * (SLIDER_SEGMENT + SLIDER_GAP) - SLIDER_GAP;
        draw_panel(canvas, meshes, panel);
        for (i, (item, kind)) in menu.items().iter().zip(rows).enumerate() {
            let mut row = menu_row_rect(panel, i, OPTION_ROW_HEIGHT);
            if let OptionRow::Back = kind {
                row.y += back_gap;
            }
            let selected = menu.selected() == i;
            let mut text = item.clone();
            if selected {
                match kind {
                    // centered words: the blocks hug the word
                    OptionRow::Back | OptionRow::Link => {
                        let size = if let OptionRow::Link = kind {
                            OPTION_LABEL_SIZE
                        } else {
                            MENU_ITEM_SIZE
                        };
                        let width = text.clone().set_scale(px(size)).measure_ui().x;
                        draw_selection_around(time, canvas, meshes, row, Some(width), style.block);
                    }
                    _ => draw_selection(time, canvas, meshes, row, style.block),
                }
            }
            let color = if selected { ACCENT_COLOR } else { Color::WHITE };
            match kind {
                OptionRow::Back | OptionRow::Link => {
                    let size = if let OptionRow::Back = kind {
                        MENU_ITEM_SIZE
                    } else {
                        OPTION_LABEL_SIZE
                    };
                    text.set_scale(px(size));
                    draw_text_centered(canvas, &text, row.center(), color)?;
                    continue;
                }
                _ => {}
            }

            // label on the left, setting on the right
            text.set_scale(px(OPTION_LABEL_SIZE));
            let size = text.measure_ui();
            let center_y = row.y + row.h / 2.0;
            draw_shadowed(
                canvas,
                &text,
                Vec2::new(row.x + 48.0, center_y - size.y / 2.0),
                color,
            );
            let right_x = row.x + row.w - 48.0;
            let arrow = SLIDER_SEGMENT * 0.6;
            let arrow_color = if selected {
                ACCENT_COLOR
            } else {
                SLIDER_EMPTY_COLOR
            };
            match kind {
                OptionRow::Choice(value, locked) => {
                    let center_x = right_x - slider_width / 2.0;
                    draw_text_centered(
                        canvas,
                        &ui_text(value, OPTION_LABEL_SIZE * 0.75),
                        Vec2::new(center_x, center_y),
                        if *locked { LABEL_COLOR } else { color },
                    )?;
                    if *locked {
                        continue;
                    }
                    // arrow blocks either side of the value
                    for x in [right_x - slider_width, right_x - arrow] {
                        let rect = Rect::new(x, center_y - arrow / 2.0, arrow, arrow);
                        draw_block(canvas, meshes, rect, arrow_color, style.block);
                    }
                }
                OptionRow::Slider(level) => draw_slider(
                    canvas,
                    meshes,
                    right_x,
                    center_y,
                    *level,
                    max_level,
                    style.block,
                ),
                OptionRow::Back | OptionRow::Link => {}
            }
        }

        // controls reference, no hold in hard mode
        let rows: Vec<_> = CONTROLS
            .iter()
            .filter(|(action, _)| mode != Mode::Hard || *action != "Hold")
            .collect();
        let header = PANEL_PADDING + CONTROLS_ROW_HEIGHT * 1.2;
        let controls = Rect::new(
            panel.x,
            panel.y + panel.h + 12.0,
            panel.w,
            header + rows.len() as f32 * CONTROLS_ROW_HEIGHT + PANEL_PADDING,
        );
        draw_panel(canvas, meshes, controls);
        let left = controls.x + PANEL_PADDING + 28.0;
        let right = controls.x + controls.w - PANEL_PADDING - 28.0;
        draw_text(
            canvas,
            "Controls",
            Vec2::new(left, controls.y + PANEL_PADDING),
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
            let width = keys.measure_ui().x;
            draw_shadowed(canvas, &keys, Vec2::new(right - width, y), Color::WHITE);
        }

        draw_hint(
            canvas,
            "Up/Down: select   Left/Right: adjust   Enter: open   Esc: back",
            view_rect,
        )
    }
}
