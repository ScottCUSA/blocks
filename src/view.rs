use macroquad::prelude::*;

use crate::board::{self, RustrisBoard, SlotState};
use crate::rustomino::Rustomino;
use crate::VIEW_DIMENSIONS;

pub(crate) const BLOCK_SIZE: i32 = 30;
pub(crate) const BLOCK_PADDING: i32 = 1;
pub(crate) const STAGING_PADDING: i32 = 2;
pub(crate) const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
pub(crate) const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const BOARD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.2);
pub(crate) const GHOST_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
pub(crate) const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
pub(crate) const VIEW_SETTINGS: ViewSettings = ViewSettings::new(VIEW_DIMENSIONS);

pub struct ViewSettings {
    pub view_w: i32,
    pub view_h: i32,
    pub board_rect: Rect<f32>,
    pub staging_rect: Rect<f32>,
    pub preview_rect: Rect<f32>,
    pub hold_rect: Rect<f32>,
    pub score_label_pos: IVec2,
    pub level_label_pos: IVec2,
    pub title_pos: IVec2,
    pub level_pos: IVec2,
    pub score_pos: IVec2,
}

impl ViewSettings {
    const fn new(view_dimensions: [i32; 2]) -> Self {
        let board_w = (board::BOARD_SLOTS[0] * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let board_h = ((board::BOARD_SLOTS[1] - 2) * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let staging_w = board_w;
        let staging_h = (2 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        let hold_w = preview_w;
        let hold_h = staging_h;

        let board_x = view_dimensions[0] / 2 - board_w / 2;
        let board_y = view_dimensions[1] / 2 - board_h / 2 + staging_h / 2 + 1;
        let staging_x = board_x;
        let staging_y = board_y - staging_h - STAGING_PADDING;
        let preview_x = board_x + board_w + 10;
        let preview_y = board_y;
        let hold_x = board_x - preview_w - 10;
        let hold_y = board_y;

        Self {
            view_w: view_dimensions[0],
            view_h: view_dimensions[1],
            board_rect: Rect::new(
                board_x as f32,
                board_y as f32,
                board_w as f32,
                board_h as f32,
            ),
            staging_rect: Rect::new(
                staging_x as f32,
                staging_y as f32,
                staging_w as f32,
                staging_h as f32,
            ),
            preview_rect: Rect::new(
                preview_x as f32,
                preview_y as f32,
                preview_w as f32,
                preview_h as f32,
            ),
            hold_rect: Rect::new(hold_x as f32, hold_y as f32, hold_w as f32, hold_h as f32),
            score_label_pos: ivec2(board_x + board_w + 30, board_y + board_h - 30),
            level_label_pos: ivec2(board_x - 180, board_y + board_h - 30),
            title_pos: ivec2(board_x - 280, board_y - 50),
            level_pos: ivec2(board_x - 60, board_y + board_h - 30),
            score_pos: ivec2(board_x + board_w + 150, board_y + board_h - 30),
        }
    }
}

pub fn draw_playing_backgound() {
    draw_rectangle(
        VIEW_SETTINGS.staging_rect.x,
        VIEW_SETTINGS.staging_rect.y,
        VIEW_SETTINGS.staging_rect.w,
        VIEW_SETTINGS.staging_rect.h,
        STAGING_BACKGROUND_COLOR,
    );

    draw_rectangle(
        VIEW_SETTINGS.board_rect.x,
        VIEW_SETTINGS.board_rect.y,
        VIEW_SETTINGS.board_rect.w,
        VIEW_SETTINGS.board_rect.h,
        BOARD_BACKGROUND_COLOR,
    );

    draw_rectangle(
        VIEW_SETTINGS.preview_rect.x,
        VIEW_SETTINGS.preview_rect.y,
        VIEW_SETTINGS.preview_rect.w,
        VIEW_SETTINGS.preview_rect.h,
        PREVIEW_BACKGROUND_COLOR,
    );

    draw_rectangle(
        VIEW_SETTINGS.hold_rect.x,
        VIEW_SETTINGS.hold_rect.y,
        VIEW_SETTINGS.hold_rect.w,
        VIEW_SETTINGS.hold_rect.h,
        HOLD_BACKGROUND_COLOR,
    );
}

pub fn draw_playing(
    board: &RustrisBoard,
    next_rustomino: &Option<Rustomino>,
    held_rustomino: &Option<Rustomino>,
) {
    for (y, slots_x) in board.slots.iter().enumerate() {
        for (x, slot) in slots_x.iter().enumerate() {
            match slot {
                SlotState::Locked(rtype) | SlotState::Occupied(rtype) => {
                    // draw the block
                    let rect = board_block_rect([x as i32, y as i32]);
                    draw_rectangle(rect.x, rect.y, rect.w, rect.h, rtype.color());
                }
                _ => {}
            }
        }
    }

    if let Some(next) = next_rustomino {
        for slot in next.blocks {
            // display the preview
            // draw the block
            let rect = next_block_rect([slot[0], slot[1]]);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, next.rustomino_type.color());
        }
    }

    if let Some(held) = held_rustomino {
        for slot in held.blocks {
            // display the preview
            // draw the block
            let rect = hold_block_rect([slot[0], slot[1]]);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, held.rustomino_type.color());
        }
    }

    if let Some(ghost) = &board.ghost_rustomino {
        for block in ghost.board_slots() {
            // draw the block
            let rect = board_block_rect([block[0], block[1]]);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 4., GHOST_COLOR);
        }
    }
}

pub fn draw_playing_overlay(text_params: &TextParams, game_level: usize, score: usize) {
    draw_text_ex(
        "Rustris",
        VIEW_SETTINGS.title_pos.x as f32,
        VIEW_SETTINGS.title_pos.y as f32,
        *text_params,
    );

    draw_text_ex(
        "Level:",
        VIEW_SETTINGS.level_label_pos.x as f32,
        VIEW_SETTINGS.level_label_pos.y as f32,
        *text_params,
    );

    draw_text_ex(
        &game_level.to_string(),
        VIEW_SETTINGS.level_pos.x as f32,
        VIEW_SETTINGS.level_pos.y as f32,
        *text_params,
    );

    draw_text_ex(
        "Score:",
        VIEW_SETTINGS.score_label_pos.x as f32,
        VIEW_SETTINGS.score_label_pos.y as f32,
        *text_params,
    );

    draw_text_ex(
        &score.to_string(),
        VIEW_SETTINGS.score_pos.x as f32,
        VIEW_SETTINGS.score_pos.y as f32,
        *text_params,
    );
}

pub fn draw_paused(text_params: &TextParams) {
    draw_rectangle(
        0.,
        0.,
        VIEW_SETTINGS.view_w as f32,
        VIEW_SETTINGS.view_h as f32,
        PAUSED_OVERLAY_COLOR,
    );
    draw_text_ex(
        "Paused",
        (VIEW_SETTINGS.view_w / 2 - 75) as f32,
        (VIEW_SETTINGS.view_h / 2) as f32,
        *text_params,
    );
}

pub fn draw_menu(text_params: &TextParams) {
    draw_rectangle(
        0.,
        0.,
        VIEW_SETTINGS.view_w as f32,
        VIEW_SETTINGS.view_h as f32,
        PAUSED_OVERLAY_COLOR,
    );
    draw_text_ex(
        "Welcome to Rustris!",
        (VIEW_SETTINGS.view_w / 2 - 230) as f32,
        (VIEW_SETTINGS.view_h / 2 - 30) as f32,
        *text_params,
    );
    draw_text_ex(
        "Press Enter To Start",
        (VIEW_SETTINGS.view_w / 2 - 253) as f32,
        (VIEW_SETTINGS.view_h / 2 + 30) as f32,
        *text_params,
    );
}

pub fn draw_gameover(text_params: &TextParams) {
    draw_rectangle(
        0.,
        0.,
        VIEW_SETTINGS.view_w as f32,
        VIEW_SETTINGS.view_h as f32,
        PAUSED_OVERLAY_COLOR,
    );
    draw_text_ex(
        "Game Over!",
        (VIEW_SETTINGS.view_w / 2 - 122) as f32,
        (VIEW_SETTINGS.view_h / 2 - 30) as f32,
        *text_params,
    );
    draw_text_ex(
        "Press Enter To Play Again",
        (VIEW_SETTINGS.view_w / 2 - 310) as f32,
        (VIEW_SETTINGS.view_h / 2 + 30) as f32,
        *text_params,
    );
}

fn next_block_rect(block: [i32; 2]) -> Rect<f32> {
    // block[x,y] absolute units
    let x = VIEW_SETTINGS.preview_rect.x
        + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        + 1.0;
    // get bottom left of board_rect
    let y = VIEW_SETTINGS.preview_rect.y + VIEW_SETTINGS.preview_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn hold_block_rect(block: [i32; 2]) -> Rect<f32> {
    // block[x,y] absolute units
    let x =
        VIEW_SETTINGS.hold_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32) + 1.0;
    // get bottom left of board_rect
    let y = VIEW_SETTINGS.hold_rect.y + VIEW_SETTINGS.hold_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn board_block_rect(block: [i32; 2]) -> Rect<f32> {
    // block[x,y] absolute units
    let x = VIEW_SETTINGS.staging_rect.x
        + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        + 1.0;
    // get bottom left of board_rect
    let y = VIEW_SETTINGS.board_rect.y + VIEW_SETTINGS.board_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        - 1.0;

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T> {
    pub const fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect { x, y, w, h }
    }
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
