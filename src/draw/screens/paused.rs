use ggez::glam::Vec2;
use ggez::graphics::Rect;
use ggez::GameResult;

use crate::menus;

use super::*;

const PAUSED_TOP: f32 = 0.36; // of the view height

/// pause menu, over the game
pub struct PausedScreen<'a>(pub &'a menus::MenuList);

impl PausedScreen<'_> {
    /// row under a point
    pub fn hit(&self, view_rect: &Rect, point: Vec2) -> Option<MenuHit> {
        menu_list_hit(view_rect, self.0, view_rect.h * PAUSED_TOP, point)
    }
}

impl Layer for PausedScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view_rect = canvas.view.view_rect;
        draw_overlay(canvas, 1.0);
        let top = view_rect.h * PAUSED_TOP;
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
