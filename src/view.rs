use macroquad::prelude::*;

use crate::board;

pub(crate) const BLOCK_SIZE: i32 = 30;
pub(crate) const BLOCK_PADDING: i32 = 1;
pub(crate) const STAGING_PADDING: i32 = 2;
pub(crate) const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
pub(crate) const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const BOARD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(crate) const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.2);
pub(crate) const GHOST_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);

pub struct ViewSettings {
    pub board_rect: Rect,
    pub staging_rect: Rect,
    pub preview_rect: Rect,
    pub hold_rect: Rect,
    pub score_label_pos: IVec2,
    pub level_label_pos: IVec2,
    pub title_label_pos: IVec2,
    pub level_pos: IVec2,
    pub score_pos: IVec2,
}

impl ViewSettings {
    pub fn new(view_dimensions: [i32; 2]) -> Self {
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
            title_label_pos: ivec2(board_x - 280, board_y - 50),
            level_pos: ivec2(board_x - 60, board_y + board_h - 30),
            score_pos: ivec2(board_x + board_w + 150, board_y + board_h - 30),
        }
    }
}
