use ggez::GameResult;

use super::*;

const DEMO_SIZE: f32 = 32.0;

/// the demo game's label, beside the board
pub struct DemoScreen;

impl Layer for DemoScreen {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let view = canvas.view;
        let (x, y) = (view.hold_rect.center().x, view.playfield_rect.center().y);
        // blinks
        if canvas.time().fract() < 0.7 {
            let demo = ui_text("Demo", DEMO_SIZE);
            draw_text_centered(canvas, &demo, Vec2::new(x, y), ACCENT_COLOR)?;
        }
        for (i, line) in ["Press", "any key"].into_iter().enumerate() {
            let hint = ui_text(line, SMALL_TEXT_SIZE);
            let y = y + DEMO_SIZE + i as f32 * SMALL_TEXT_SIZE * 1.5;
            draw_text_centered(canvas, &hint, Vec2::new(x, y), LABEL_COLOR)?;
        }
        Ok(())
    }
}
