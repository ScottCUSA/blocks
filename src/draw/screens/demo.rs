use ggez::GameResult;

use crate::draw::*;

const DEMO_SIZE: f32 = 32.0;

/// the demo game's label, beside the board
pub struct DemoScreen;

impl Layer for DemoScreen {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let ctx = frame.ctx;
        let view = frame.view;
        let x = view.hold_rect.x + view.hold_rect.w / 2.0;
        let y = view.playfield_rect.y + view.playfield_rect.h / 2.0;
        // blinks
        if ctx.time.time_since_start().as_secs_f32().fract() < 0.7 {
            let demo = ui_text("Demo", DEMO_SIZE);
            draw_text_centered(ctx, canvas, &demo, Vec2::new(x, y), ACCENT_COLOR)?;
        }
        for (i, line) in ["Press", "any key"].into_iter().enumerate() {
            let hint = ui_text(line, SMALL_TEXT_SIZE);
            let y = y + DEMO_SIZE + i as f32 * SMALL_TEXT_SIZE * 1.5;
            draw_text_centered(ctx, canvas, &hint, Vec2::new(x, y), LABEL_COLOR)?;
        }
        Ok(())
    }
}
