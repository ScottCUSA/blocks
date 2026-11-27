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
pub(super) const OPTIONS_TOP: f32 = 0.19; // of the view height
pub(super) const VALUE_INSET: f32 = 48.0; // px from the row ends to the label and value
pub(super) const BUTTON_REACH: f32 = 24.0; // px a value's click targets reach past it
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
    pub rows: Vec<OptionRow<'a>>,
    pub mode: Mode,
    pub max_level: u8,
    pub hover: Option<Hover>,
}

impl OptionsScreen<'_> {
    /// settings panel, a little space above Back so it reads as its own button
    fn panel(&self, view_rect: &Rect) -> Rect {
        let (top, count) = (view_rect.h * OPTIONS_TOP, self.menu.items.len());
        let mut panel = menu_panel_rect(
            view_rect,
            count,
            top,
            OPTIONS_PANEL_WIDTH,
            OPTION_ROW_HEIGHT,
        );
        panel.h += BACK_GAP;
        panel
    }

    fn row_rect(&self, panel: Rect, i: usize) -> Rect {
        let mut row = menu_row_rect(panel, i, OPTION_ROW_HEIGHT);
        if let Some(OptionRow::Back) = self.rows.get(i) {
            row.y += BACK_GAP;
        }
        row
    }

    /// click targets on a row's value, and what each sets
    fn buttons(&self, rect: Rect, kind: &OptionRow) -> Vec<(Rect, RowPart)> {
        let max = self.max_level;
        let right_x = rect.right() - VALUE_INSET;
        let left_x = right_x - slider_width(max);
        let span = |x0: f32, x1: f32| Rect::new(x0, rect.y, x1 - x0, rect.h);
        match kind {
            // a half of the value each
            OptionRow::Choice(_, false) => {
                let center_x = (left_x + right_x) / 2.0;
                vec![
                    (span(left_x - BUTTON_REACH, center_x), RowPart::Step(-1)),
                    (span(center_x, right_x + BUTTON_REACH), RowPart::Step(1)),
                ]
            }
            // split mid gap
            OptionRow::Slider(_) => {
                let center_y = rect.center().y;
                let cells: Vec<_> = (0..=max)
                    .map(|level| slider_cell(right_x, center_y, max, level))
                    .collect();
                let count = cells.len();
                let edge = |i: usize| match i {
                    0 => cells[0].left() - BUTTON_REACH,
                    i if i == count => cells[count - 1].right() + BUTTON_REACH,
                    i => (cells[i - 1].right() + cells[i].left()) / 2.0,
                };
                (0..count)
                    .map(|i| (span(edge(i), edge(i + 1)), RowPart::Level(i as u8)))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// row and part under a point
    pub fn hit(&self, view_rect: &Rect, point: Vec2) -> Option<MenuHit> {
        let panel = self.panel(view_rect);
        let row = (0..self.rows.len()).find(|&i| self.row_rect(panel, i).contains(point))?;
        let part = self
            .buttons(self.row_rect(panel, row), &self.rows[row])
            .into_iter()
            .find(|(button, _)| button.contains(point))
            .map_or(RowPart::Row, |(_, part)| part);
        Some(MenuHit { row, part })
    }
}

impl Layer for OptionsScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let Self {
            title,
            menu,
            ref rows,
            mode,
            max_level,
            hover,
        } = *self;
        let view_rect = canvas.view.view_rect;
        draw_title(canvas, title, view_rect.h * 0.12, TITLE_SIZE)?;

        // settings, sliders and back
        let panel = self.panel(&view_rect);
        let slider_width = slider_width(max_level);
        draw_panel(canvas, panel);
        for (i, (item, kind)) in menu.items.iter().zip(rows).enumerate() {
            let row = self.row_rect(panel, i);
            let selected = menu.selected == Some(i);
            let hovered = hover
                .filter(|hover| hover.hit.row == i && selected)
                .map(|hover| (hover.hit.part, hover.since));
            let color = if selected { ACCENT_COLOR } else { Color::WHITE };
            // centered words, the blocks hug the word
            let centered = match kind {
                OptionRow::Back => Some(MENU_ITEM_SIZE),
                OptionRow::Link => Some(OPTION_LABEL_SIZE),
                _ => None,
            };
            if let Some(size) = centered {
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
            draw_shadowed(
                canvas,
                &text,
                Vec2::new(row.x + VALUE_INSET, label_y),
                color,
            );
            let right_x = row.x + row.w - VALUE_INSET;
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
                    // arrow blocks either side of the value, a hovered one alone lit and pulsing
                    let on_arrow = matches!(hovered, Some((RowPart::Step(_), _)));
                    for (step, x) in [(-1, right_x - slider_width), (1, right_x - arrow)] {
                        let mut rect = Rect::new(x, center_y - arrow / 2.0, arrow, arrow);
                        let mut color = if on_arrow {
                            SLIDER_EMPTY_COLOR
                        } else {
                            arrow_color
                        };
                        if let Some((_, since)) =
                            hovered.filter(|&(part, _)| part == RowPart::Step(step))
                        {
                            rect = hover_pulse(canvas, rect, since);
                            color = ACCENT_COLOR;
                        }
                        draw_block(canvas, rect, color);
                    }
                }
                OptionRow::Slider(level) => {
                    let hot = match hovered {
                        Some((RowPart::Level(level), since)) => Some((level, since)),
                        _ => None,
                    };
                    draw_slider(canvas, right_x, center_y, *level, max_level, hot)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_parts() {
        let view = ViewSettings::new(VIEW_WIDTH, VIEW_HEIGHT);
        let rect = &view.view_rect;
        let menu = menus::MenuList::options();
        let screen = OptionsScreen {
            title: "Options",
            menu: &menu,
            rows: vec![
                OptionRow::Choice("Standard", false),
                OptionRow::Choice("Stars", true),
                OptionRow::Slider(3),
                OptionRow::Back,
            ],
            mode: Mode::Standard,
            max_level: 10,
            hover: None,
        };
        let panel = screen.panel(rect);
        let right_x = screen.row_rect(panel, 0).right() - VALUE_INSET;
        let left_x = right_x - slider_width(10);
        let at = |row, x| {
            let y = screen.row_rect(panel, row).center().y;
            screen
                .hit(rect, Vec2::new(x, y))
                .map(|hit| (hit.row, hit.part))
        };
        let reach = BUTTON_REACH - 1.0;
        assert_eq!(at(0, left_x - reach), Some((0, RowPart::Step(-1))));
        assert_eq!(at(0, right_x + reach), Some((0, RowPart::Step(1))));
        assert_eq!(at(0, panel.x + VALUE_INSET), Some((0, RowPart::Row)));
        // locked
        assert_eq!(at(1, right_x - 1.0), Some((1, RowPart::Row)));
        let pitch = SLIDER_SEGMENT + SLIDER_GAP;
        for level in 1..=10 {
            let x = left_x + (level - 1) as f32 * pitch + SLIDER_SEGMENT / 2.0;
            assert_eq!(at(2, x), Some((2, RowPart::Level(level))));
        }
        let off = slider_cell(right_x, 0.0, 10, 0);
        assert_eq!(at(2, off.center().x), Some((2, RowPart::Level(0))));
        assert_eq!(at(2, off.left() - reach), Some((2, RowPart::Level(0))));
        // mid gap between off and 1
        let off_gap_x = (off.right() + left_x) / 2.0;
        assert_eq!(at(2, off_gap_x - 0.5), Some((2, RowPart::Level(0))));
        assert_eq!(at(2, off_gap_x + 0.5), Some((2, RowPart::Level(1))));
        assert_eq!(at(2, right_x + reach), Some((2, RowPart::Level(10))));
        // mid gap between 1 and 2
        let gap_x = left_x + SLIDER_SEGMENT + SLIDER_GAP / 2.0;
        assert_eq!(at(2, gap_x - 0.5), Some((2, RowPart::Level(1))));
        assert_eq!(at(2, gap_x + 0.5), Some((2, RowPart::Level(2))));
        assert_eq!(at(3, panel.center().x), Some((3, RowPart::Row)));
        // the gap above Back
        let back = screen.row_rect(panel, 3);
        let gap = Vec2::new(back.center().x, back.y - BACK_GAP / 2.0);
        assert_eq!(screen.hit(rect, gap), None);
    }
}
