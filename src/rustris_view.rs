use piston_window::{types::Vec2d, Context, G2d};

use crate::rustris_controller::RustrisController;

pub struct RustrisBoardViewSettings {
    pub board_size: Vec2d,
    pub staging_size: Vec2d,
    pub preview_size: Vec2d,
}

impl Default for RustrisBoardViewSettings {
    fn default() -> Self {
        Self {
            board_size: [313.0, 623.0],
            staging_size: [65.0, 313.0],
            preview_size: [65.0, 127.0],
        }
    }
}

pub struct RustrisView {}
impl RustrisView {
    pub fn new() -> Self {
        RustrisView {}
    }
    pub fn draw(&self, controller: &RustrisController, ctx: &Context, g: &mut G2d) {
        use piston_window::Rectangle;
        // display the rustris board
        Rectangle::new([1.0, 1.0, 1.0, 1.0]).draw(
            [20.0, 20.0, 400.0, 400.0],
            &ctx.draw_state,
            ctx.transform,
            g,
        );
    }
}
