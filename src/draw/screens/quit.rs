use ggez::glam::Vec2;
use ggez::graphics::Rect;
use ggez::GameResult;

use super::*;

const QUIT_PANEL_HEIGHT: f32 = 160.0;
const QUIT_BUTTON_SIZE: Vec2 = Vec2::new(140.0, 50.0);
const QUIT_BUTTON_OFFSET: f32 = 90.0; // from the panel's center to a button's center

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitAnswer {
    Yes,
    No,
}

impl QuitAnswer {
    fn label(self) -> &'static str {
        match self {
            QuitAnswer::Yes => "Y",
            QuitAnswer::No => "N",
        }
    }

    fn rect(self, view_rect: &Rect) -> Rect {
        let panel = centered_panel(view_rect, QUIT_PANEL_HEIGHT);
        let side = match self {
            QuitAnswer::Yes => -1.0,
            QuitAnswer::No => 1.0,
        };
        let center = Vec2::new(
            panel.center().x + side * QUIT_BUTTON_OFFSET,
            panel.y + panel.h * 0.7,
        );
        Rect::new(
            center.x - QUIT_BUTTON_SIZE.x / 2.0,
            center.y - QUIT_BUTTON_SIZE.y / 2.0,
            QUIT_BUTTON_SIZE.x,
            QUIT_BUTTON_SIZE.y,
        )
    }
}

/// "quit?" prompt over the menu, Y and N can be clicked or selected by key
pub struct QuitScreen {
    pub selected: Option<QuitAnswer>,
}

impl QuitScreen {
    /// answer under a point
    pub fn hit(view_rect: &Rect, point: Vec2) -> Option<QuitAnswer> {
        [QuitAnswer::Yes, QuitAnswer::No]
            .into_iter()
            .find(|answer| answer.rect(view_rect).contains(point))
    }
}

impl Layer for QuitScreen {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view_rect = canvas.view.view_rect;
        draw_overlay(canvas, 1.0);
        let panel = centered_panel(&view_rect, QUIT_PANEL_HEIGHT);
        draw_panel(canvas, panel);
        let question = ui_text("Are You Sure?", MENU_ITEM_SIZE);
        let at = Vec2::new(panel.center().x, panel.y + panel.h * 0.3);
        draw_text_centered(canvas, &question, at, Color::WHITE)?;
        let slash = ui_text("/", MENU_ITEM_SIZE);
        let at = Vec2::new(panel.center().x, panel.y + panel.h * 0.7);
        draw_text_centered(canvas, &slash, at, Color::WHITE)?;
        for answer in [QuitAnswer::Yes, QuitAnswer::No] {
            let rect = answer.rect(&view_rect);
            let selected = self.selected == Some(answer);
            let text = ui_text(answer.label(), MENU_ITEM_SIZE);
            if selected {
                let width = text.measure_ui(canvas)?.x;
                draw_selection_around(canvas, rect, Some(width));
            }
            let color = if selected { ACCENT_COLOR } else { Color::WHITE };
            draw_text_centered(canvas, &text, rect.center().into(), color)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_answers() {
        let view = ViewSettings::new(VIEW_WIDTH, VIEW_HEIGHT);
        let rect = &view.view_rect;
        for answer in [QuitAnswer::Yes, QuitAnswer::No] {
            let center = answer.rect(rect).center().into();
            assert_eq!(QuitScreen::hit(rect, center), Some(answer));
        }
        let panel = centered_panel(rect, QUIT_PANEL_HEIGHT);
        assert_eq!(QuitScreen::hit(rect, panel.center().into()), None);
    }
}
