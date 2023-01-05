use macroquad::prelude::*;

use crate::board;

pub(crate) const BLOCK_SIZE: f32 = 30.0;
pub(crate) const BLOCK_PADDING: f32 = 1.0;
pub(crate) const STAGING_PADDING: f32 = 2.0;
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
    pub score_label_pos: Vec2,
    pub level_label_pos: Vec2,
    pub title_label_pos: Vec2,
    pub level_pos: Vec2,
    pub score_pos: Vec2,
}

impl ViewSettings {
    pub fn new(view_dimensions: [f32; 2]) -> Self {
        let board_w = (board::BOARD_SLOTS[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let board_h =
            ((board::BOARD_SLOTS[1] as f32 - 2.0) * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let staging_w = board_w;
        let staging_h = (2.0 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4.0 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        let hold_w = preview_w;
        let hold_h = staging_h;

        let board_x = view_dimensions[0] / 2.0 - board_w / 2.0;
        let board_y = view_dimensions[1] / 2.0 - board_h / 2.0 + staging_h / 2.0 + 1.0;
        let staging_x = board_x;
        let staging_y = board_y - staging_h - STAGING_PADDING;
        let preview_x = board_x + board_w + 10.0;
        let preview_y = board_y;
        let hold_x = board_x - preview_w - 10.0;
        let hold_y = board_y;

        Self {
            board_rect: Rect::new(board_x, board_y, board_w, board_h),
            staging_rect: Rect::new(staging_x, staging_y, staging_w, staging_h),
            preview_rect: Rect::new(preview_x, preview_y, preview_w, preview_h),
            hold_rect: Rect::new(hold_x, hold_y, hold_w, hold_h),
            score_label_pos: [board_x + board_w + 30.0, board_y + board_h - 30.0].into(),
            level_label_pos: [board_x - 180.0, board_y + board_h - 30.0].into(),
            title_label_pos: [board_x - 280.0, board_y - 50.0].into(),
            level_pos: [board_x - 60.0, board_y + board_h - 30.0].into(),
            score_pos: [board_x + board_w + 150.0, board_y + board_h - 30.0].into(),
        }
    }
}
