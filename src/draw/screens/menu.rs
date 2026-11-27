use ggez::GameResult;

use crate::menus;

use super::*;

/// main menu
pub struct MenuScreen<'a>(pub &'a menus::MenuList);

impl Layer for MenuScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view_rect = canvas.view.view_rect;
        draw_title(canvas, "Blocks!", view_rect.h * 0.16, TITLE_SIZE)?;
        draw_menu_list(canvas, self.0, view_rect.h * 0.42)?;
        draw_hint(canvas, "Up/Down: select   Enter: choose")
    }
}
