use ggez::glam::Vec2;
use ggez::graphics::Rect;
use ggez::GameResult;

use crate::menus;

use super::*;

const MENU_TOP: f32 = 0.42; // of the view height

/// main menu
pub struct MenuScreen<'a>(pub &'a menus::MenuList);

impl MenuScreen<'_> {
    /// row under a point
    pub fn hit(&self, view_rect: &Rect, point: Vec2) -> Option<MenuHit> {
        menu_list_hit(view_rect, self.0, view_rect.h * MENU_TOP, point)
    }
}

impl Layer for MenuScreen<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view_rect = canvas.view.view_rect;
        draw_title(canvas, "Blocks!", view_rect.h * 0.16, TITLE_SIZE)?;
        draw_menu_list(canvas, self.0, view_rect.h * MENU_TOP)?;
        draw_hint(canvas, "Up/Down: select   Enter: choose")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_rows() {
        let view = ViewSettings::new(VIEW_WIDTH, VIEW_HEIGHT);
        let rect = &view.view_rect;
        let menu = menus::MenuList::main();
        let screen = MenuScreen(&menu);
        let panel = menu_list_panel(rect, &menu, rect.h * MENU_TOP);
        for row in 0..menu.items.len() {
            let center = menu_row_rect(panel, row, MENU_ROW_HEIGHT).center().into();
            let part = RowPart::Row;
            assert_eq!(screen.hit(rect, center), Some(MenuHit { row, part }));
        }
        assert_eq!(screen.hit(rect, Vec2::new(panel.x, panel.y) - 1.0), None);
    }
}
