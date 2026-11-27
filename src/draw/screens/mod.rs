use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::draw::*;
use crate::menus::MenuList;

mod debug;
mod demo;
mod game_over;
mod high_scores;
mod menu;
mod name_entry;
mod options;
mod paused;

pub use debug::{DebugScreen, FrameStats};
pub use demo::DemoScreen;
pub use game_over::GameOverScreen;
pub use high_scores::HighScoresScreen;
pub use menu::MenuScreen;
pub use name_entry::NameEntryScreen;
pub use options::{OptionRow, OptionsScreen};
pub use paused::PausedScreen;

pub(super) const TITLE_SIZE: f32 = 96.0;
pub(super) const MENU_PANEL_WIDTH: f32 = 520.0;
pub(super) const PAUSED_TITLE_SIZE: f32 = 64.0;

/// a MENU_PANEL_WIDTH panel centered in the view
pub(super) fn centered_panel(view_rect: &Rect, height: f32) -> Rect {
    let (x, y) = (
        (view_rect.w - MENU_PANEL_WIDTH) / 2.0,
        (view_rect.h - height) / 2.0,
    );
    Rect::new(x, y, MENU_PANEL_WIDTH, height)
}

/// a menu in a panel centered across the view
pub(super) fn draw_menu_list(canvas: &mut Painter, menu: &MenuList, top: f32) -> GameResult {
    let rows = menu.items.len();
    let view_rect = canvas.view.view_rect;
    let panel = menu_panel_rect(&view_rect, rows, top, MENU_PANEL_WIDTH, MENU_ROW_HEIGHT);
    draw_panel(canvas, panel);
    for (i, item) in menu.items.iter().enumerate() {
        let row = menu_row_rect(panel, i, MENU_ROW_HEIGHT);
        let selected = menu.selected() == i;
        let text = ui_text(item, MENU_ITEM_SIZE);
        if selected {
            let width = text.measure_ui(canvas)?.x;
            draw_selection_around(canvas, row, Some(width));
        }
        let color = if selected { ACCENT_COLOR } else { Color::WHITE };
        draw_text_centered(canvas, &text, row.center().into(), color)?;
    }
    Ok(())
}
