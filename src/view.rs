use crate::{
    board::{RustrisBoard, PLAYFIELD_SIZE},
    rustomino::{Rustomino, RustominoType},
};
use piston_window::{types::Color, Context, G2d, ResizeArgs};

use crate::controller::RustrisController;

const BLOCK_SIZE: i32 = 30;
const BLOCK_PADDING: i32 = 1;
const STAGING_PADDING: i32 = 2;

// generic rect implementation supporting
// conversion from and to [T; 4]
#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> From<[T; 4]> for Rect<T>
where
    T: Copy,
{
    fn from(value: [T; 4]) -> Self {
        Rect {
            x: value[0],
            y: value[1],
            w: value[2],
            h: value[3],
        }
    }
}

impl<T> From<Rect<T>> for [T; 4] {
    fn from(value: Rect<T>) -> Self {
        [value.x, value.y, value.w, value.h]
    }
}

pub struct ViewSettings {
    pub board_rect: Rect<f64>,
    pub staging_rect: Rect<f64>,
    pub preview_rect: Rect<f64>,
}

impl ViewSettings {
    fn new(view_size: [u32; 2]) -> Self {
        let board_w = (PLAYFIELD_SIZE[0] as i32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let board_h =
            ((PLAYFIELD_SIZE[1] as i32 - 2) * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let staging_w = board_w;
        let staging_h = (2 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;

        let board_x = view_size[0] as i32 / 2 - board_w / 2;
        let board_y = view_size[1] as i32 / 2 - board_h / 2 + staging_h / 2 + 1;
        let staging_x = board_x;
        let staging_y = board_y - staging_h - STAGING_PADDING;
        let preview_x = board_x + board_w + 10;
        let preview_y = board_y;

        Self {
            board_rect: [
                board_x as f64,
                board_y as f64,
                board_w as f64,
                board_h as f64,
            ]
            .into(),
            staging_rect: [
                staging_x as f64,
                staging_y as f64,
                staging_w as f64,
                staging_h as f64,
            ]
            .into(),
            preview_rect: [
                preview_x as f64,
                preview_y as f64,
                preview_w as f64,
                preview_h as f64,
            ]
            .into(),
        }
    }
}

pub trait Draw {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut G2d);
}

pub struct RustrisView {
    settings: ViewSettings,
}

impl RustrisView {
    pub fn new(view_size: [u32; 2]) -> Self {
        RustrisView {
            settings: ViewSettings::new(view_size),
        }
    }
    pub fn resize(&mut self, args: ResizeArgs) {
        self.settings = ViewSettings::new(args.draw_size);
    }
    pub fn draw(&self, controller: &RustrisController, ctx: &Context, g: &mut G2d) {
        match controller.game_state {
            crate::controller::GameState::Menu => {}
            crate::controller::GameState::Playing => {
                controller.board.draw(&self.settings, ctx, g);
            }
            crate::controller::GameState::GameOver => {}
        }
        // probably want to implement game states
        // and draw them appropriately
        // menu, playing, gameover, etc

        // playing game state would be
        // display the rustris board
        // display the score
        // display the level
    }
}

impl Draw for RustrisBoard {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut G2d) {
        use piston_window::Rectangle;

        // display the board background
        Rectangle::new([0.0, 0.0, 0.0, 0.5]).draw(
            settings.board_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        // display the staging area
        Rectangle::new([0.0, 0.0, 0.0, 0.5]).draw(
            settings.staging_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        // display the preview
        Rectangle::new([0.0, 0.0, 0.0, 0.5]).draw(
            settings.preview_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        for rustomino in self.current_rustomino.iter() {
            rustomino.draw(settings, ctx, g);
        }

        for (y, slots_x) in self.slots.iter().enumerate() {
            for (x, slot) in slots_x.iter().enumerate() {
                match slot {
                    crate::board::SlotState::Locked(rtype) => {
                        // draw the block
                        Rectangle::new(rustomino_color(*rtype)).draw(
                            block_rect([x as i32, y as i32], settings),
                            &ctx.draw_state,
                            ctx.transform,
                            g,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

const I_COLOR: Color = [0.0, 0.9, 1.0, 1.0]; // light blue
const O_COLOR: Color = [1.0, 0.87, 0.0, 1.0]; // yellow
const T_COLOR: Color = [0.72, 0.01, 0.99, 1.0]; // purple
const L_COLOR: Color = [1.0, 0.45, 0.03, 1.0]; // orange
const J_COLOR: Color = [0.09, 0.0, 1.0, 1.0]; // blue
const S_COLOR: Color = [0.4, 0.99, 0.0, 1.0]; // green
const Z_COLOR: Color = [1.0, 0.06, 0.24, 1.0]; // red

fn rustomino_color(rtype: RustominoType) -> Color {
    match rtype {
        RustominoType::I => I_COLOR,
        RustominoType::O => O_COLOR,
        RustominoType::T => T_COLOR,
        RustominoType::L => L_COLOR,
        RustominoType::J => J_COLOR,
        RustominoType::S => S_COLOR,
        RustominoType::Z => Z_COLOR,
    }
}

fn block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect<f64> {
    // block[x,y] absolute units
    let x = settings.staging_rect.x + (block[0] * (BLOCK_SIZE + BLOCK_PADDING)) as f64 + 1.0;
    // get bottom left of board_rect
    let y = settings.board_rect.y + settings.board_rect.h
        - ((block[1] + 1) * (BLOCK_SIZE + BLOCK_PADDING)) as f64
        - 1.0;

    [x, y, BLOCK_SIZE as f64, BLOCK_SIZE as f64].into()
}

impl Draw for Rustomino {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut G2d) {
        use piston_window::Rectangle;
        for block in self.block_slots() {
            // display the preview
            Rectangle::new(rustomino_color(self.rustomino_type)).draw(
                block_rect(block, settings),
                &ctx.draw_state,
                ctx.transform,
                g,
            );
        }
    }
}
