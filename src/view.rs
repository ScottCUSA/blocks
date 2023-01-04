use std::path::PathBuf;

use crate::controller::RustrisController;
use crate::{
    board::{RustrisBoard, SLOTS_AREA},
    rustomino::{Rustomino, RustominoType},
};
use opengl_graphics::{GlGraphics, GlyphCache};
use piston_window::types::Vec2d;
use piston_window::{clear, text, types::Color, Context, ResizeArgs, TextureSettings};
use piston_window::{Size, Transformed};

const BLOCK_SIZE: f64 = 30.0;
const BLOCK_PADDING: f64 = 1.0;
const STAGING_PADDING: f64 = 2.0;
const BACKGROUND_COLOR: Color = [0.0, 0.29, 0.38, 1.0];
const STAGING_BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 0.5];
const BOARD_BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 0.5];
const PREVIEW_BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 0.5];
const HOLD_BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 0.2];
const GHOST_COLOR: Color = [0.7, 0.7, 0.7, 1.0];

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
    pub hold_rect: Rect<f64>,
    pub score_label_pos: Vec2d,
    pub level_label_pos: Vec2d,
    pub title_label_pos: Vec2d,
    pub level_pos: Vec2d,
    pub score_pos: Vec2d,
}

impl ViewSettings {
    fn new(view_size: Size) -> Self {
        let board_w = (SLOTS_AREA[0] as f64 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let board_h = ((SLOTS_AREA[1] as f64 - 2.0) * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let staging_w = board_w;
        let staging_h = (2.0 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4.0 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        let hold_w = preview_w;
        let hold_h = staging_h;

        let board_x = view_size.width / 2.0 - board_w / 2.0;
        let board_y = view_size.height / 2.0 - board_h / 2.0 + staging_h / 2.0 + 1.0;
        let staging_x = board_x;
        let staging_y = board_y - staging_h - STAGING_PADDING;
        let preview_x = board_x + board_w + 10.0;
        let preview_y = board_y;
        let hold_x = board_x - preview_w - 10.0;
        let hold_y = board_y;

        Self {
            board_rect: [board_x, board_y, board_w, board_h].into(),
            staging_rect: [staging_x, staging_y, staging_w, staging_h].into(),
            preview_rect: [preview_x, preview_y, preview_w, preview_h].into(),
            hold_rect: [hold_x, hold_y, hold_w, hold_h].into(),
            score_label_pos: [board_x + board_w + 30.0, board_y + board_h - 30.0],
            level_label_pos: [board_x - 180.0, board_y + board_h - 30.0],
            title_label_pos: [board_x - 280.0, board_y - 50.0],
            level_pos: [board_x - 60.0, board_y + board_h - 30.0],
            score_pos: [board_x + board_w + 150.0, board_y + board_h - 30.0],
        }
    }
}

pub trait Draw {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut GlGraphics);
}

pub struct RustrisView<'a> {
    settings: ViewSettings,
    glyph_cache: GlyphCache<'a>,
}

impl<'a> RustrisView<'a> {
    pub fn new(view_size: Size, assets_path: &PathBuf) -> Self {
        let font = assets_path.join("04b30.ttf");
        let glyph_cache = GlyphCache::new(font, (), TextureSettings::new()).unwrap();

        RustrisView {
            settings: ViewSettings::new(view_size),
            glyph_cache,
        }
    }

    pub fn resize(&mut self, args: ResizeArgs) {
        self.settings = ViewSettings::new(args.draw_size.into());
    }

    pub fn draw(&mut self, controller: &RustrisController, ctx: &Context, g: &mut GlGraphics) {
        clear(BACKGROUND_COLOR, g);

        match controller.game_state {
            crate::controller::GameState::Menu => {}
            crate::controller::GameState::Playing => {
                self.draw_playing_background(ctx, g);
                self.draw_playing_foreground(controller, ctx, g);
                self.draw_overlay(controller, ctx, g);
            }
            crate::controller::GameState::GameOver => {}
        }
        // playing game state would be
        // display the rustris board
        // display the score
        // display the level
    }

    fn draw_overlay(&mut self, controller: &RustrisController, ctx: &Context, g: &mut GlGraphics) {
        text(
            [1.0, 1.0, 1.0, 1.0],
            18,
            "Rustris",
            &mut self.glyph_cache,
            ctx.transform.trans(
                self.settings.title_label_pos[0],
                self.settings.title_label_pos[1],
            ),
            g,
        )
        .expect("unable to render text");

        text(
            [1.0, 1.0, 1.0, 1.0],
            18,
            "Level:",
            &mut self.glyph_cache,
            ctx.transform.trans(
                self.settings.level_label_pos[0],
                self.settings.level_label_pos[1],
            ),
            g,
        )
        .expect("unable to render text");

        text(
            [1.0, 1.0, 1.0, 1.0],
            18,
            &controller.game_level.to_string(),
            &mut self.glyph_cache,
            ctx.transform
                .trans(self.settings.level_pos[0], self.settings.level_pos[1]),
            g,
        )
        .expect("unable to render text");

        text(
            [1.0, 1.0, 1.0, 1.0],
            18,
            "Score:",
            &mut self.glyph_cache,
            ctx.transform.trans(
                self.settings.score_label_pos[0],
                self.settings.score_label_pos[1],
            ),
            g,
        )
        .expect("unable to render text");

        text(
            [1.0, 1.0, 1.0, 1.0],
            18,
            &controller.score.to_string(),
            &mut self.glyph_cache,
            ctx.transform
                .trans(self.settings.score_pos[0], self.settings.score_pos[1]),
            g,
        )
        .expect("unable to render text");
    }

    fn draw_playing_foreground(
        &self,
        controller: &RustrisController,
        ctx: &Context,
        g: &mut GlGraphics,
    ) {
        use piston_window::Rectangle;

        // draw the board state
        controller.board.draw(&self.settings, ctx, g);

        // draw next rustomino
        if let Some(rustomino) = controller.next_rustomino.as_ref() {
            for block in rustomino.blocks {
                // piece hold background
                Rectangle::new(rustomino_color(rustomino.rustomino_type)).draw(
                    next_block_rect(block, &self.settings),
                    &ctx.draw_state,
                    ctx.transform,
                    g,
                );
            }
        }

        // draw held rustomino
        if let Some(rustomino) = controller.hold_rustomino.as_ref() {
            for block in rustomino.blocks {
                // piece hold background
                Rectangle::new(rustomino_color(rustomino.rustomino_type)).draw(
                    hold_block_rect(block, &self.settings),
                    &ctx.draw_state,
                    ctx.transform,
                    g,
                );
            }
        }
    }

    fn draw_playing_background(&self, ctx: &Context, g: &mut GlGraphics) {
        use piston_window::Rectangle;

        // staging area background
        Rectangle::new(STAGING_BACKGROUND_COLOR).draw(
            self.settings.staging_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        // board background
        Rectangle::new(BOARD_BACKGROUND_COLOR).draw(
            self.settings.board_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        // piece preview background
        Rectangle::new(PREVIEW_BACKGROUND_COLOR).draw(
            self.settings.preview_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );

        // piece hold background
        Rectangle::new(HOLD_BACKGROUND_COLOR).draw(
            self.settings.hold_rect,
            &ctx.draw_state,
            ctx.transform,
            g,
        );
    }
}

impl Draw for RustrisBoard {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut GlGraphics) {
        use piston_window::{rectangle::Border, Rectangle};

        for (y, slots_x) in self.slots.iter().enumerate() {
            for (x, slot) in slots_x.iter().enumerate() {
                match slot {
                    crate::board::SlotState::Locked(rtype) => {
                        // draw the block
                        Rectangle::new(rustomino_color(*rtype)).draw(
                            board_block_rect([x as i32, y as i32], settings),
                            &ctx.draw_state,
                            ctx.transform,
                            g,
                        );
                    }
                    _ => {}
                }
            }
        }

        for rustomino in self.current_rustomino.iter() {
            rustomino.draw(settings, ctx, g);
        }

        for ghost in self.ghost_rustomino.iter() {
            for block in ghost.board_slots() {
                // draw the ghost block
                Rectangle::new([0.0, 0.0, 0.0, 0.0])
                    .border(Border {
                        color: GHOST_COLOR,
                        radius: 1.0,
                    })
                    .draw(
                        board_block_rect([block[0], block[1]], settings),
                        &ctx.draw_state,
                        ctx.transform,
                        g,
                    );
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

fn next_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect<f64> {
    // block[x,y] absolute units
    let x = settings.preview_rect.x + (block[0] as f64 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of board_rect
    let y = settings.preview_rect.y + settings.preview_rect.h
        - (block[1] as f64 * (BLOCK_SIZE + BLOCK_PADDING));

    [x, y, BLOCK_SIZE, BLOCK_SIZE].into()
}

fn hold_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect<f64> {
    // block[x,y] absolute units
    let x = settings.hold_rect.x + (block[0] as f64 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of board_rect
    let y = settings.hold_rect.y + settings.hold_rect.h
        - (block[1] as f64 * (BLOCK_SIZE + BLOCK_PADDING));

    [x, y, BLOCK_SIZE, BLOCK_SIZE].into()
}

fn board_block_rect(block: [i32; 2], settings: &ViewSettings) -> Rect<f64> {
    // block[x,y] absolute units
    let x = settings.staging_rect.x + (block[0] as f64 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of board_rect
    let y = settings.board_rect.y + settings.board_rect.h
        - ((block[1] + 1) as f64 * (BLOCK_SIZE + BLOCK_PADDING))
        - 1.0;

    [x, y, BLOCK_SIZE, BLOCK_SIZE].into()
}

impl Draw for Rustomino {
    fn draw(&self, settings: &ViewSettings, ctx: &Context, g: &mut GlGraphics) {
        use piston_window::Rectangle;
        for block in self.board_slots() {
            // display the preview
            Rectangle::new(rustomino_color(self.rustomino_type)).draw(
                board_block_rect(block, settings),
                &ctx.draw_state,
                ctx.transform,
                g,
            );
        }
    }
}
