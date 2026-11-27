use ggez::GameResult;

use crate::menus;

use super::*;

/// pause menu, over the game
pub struct PausedScreen<'a>(pub &'a menus::MenuList);

impl Layer for PausedScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view_rect = canvas.view.view_rect;
        draw_overlay(canvas, 1.0);
        let top = view_rect.h * 0.36;
        draw_title(
            canvas,
            "Paused",
            top - PAUSED_TITLE_SIZE * 0.75,
            PAUSED_TITLE_SIZE,
        )?;
        draw_menu_list(canvas, self.0, top)?;
        draw_hint(canvas, "Up/Down: select   Enter: choose   Esc: resume")
    }
}
