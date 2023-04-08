use ggez::glam::{IVec2, Vec2};
use ggez::graphics::{self, Canvas, Color, DrawMode, Rect};
use ggez::{Context, GameResult};
use once_cell::sync::Lazy;

use crate::game::{self, RustrisGame};
use crate::playfield::{self, RustrisPlayfield, SlotState};
use crate::rustomino::{Rustomino, RustominoType};

const BLOCK_SIZE: f32 = 30.;
const BLOCK_PADDING: f32 = 1.;
const STAGING_PADDING: f32 = 2.;

pub const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
pub const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.2);
const GHOST_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
pub static VIEW_SETTINGS: Lazy<ViewSettings> = Lazy::new(|| ViewSettings::new());
const CONTROLS_BACKGROUND_COLOR: Color = Color::new(0.34, 0.09, 0.12, 0.8);
const VIEW_WH: [f32; 2] = [1024., 768.];

#[derive(Debug)]
pub struct ViewSettings {
    pub view_w: f32,
    pub view_h: f32,
    pub view_rect: Rect,
    pub playfield_rect: Rect,
    pub staging_rect: Rect,
    pub preview_rect: Rect,
    pub hold_rect: Rect,
    pub score_label_pos: Vec2,
    pub level_label_pos: Vec2,
    pub title_pos: Vec2,
    pub level_pos: Vec2,
    pub score_pos: Vec2,
}

impl ViewSettings {
    fn new() -> Self {
        let playfield_w =
            (playfield::PLAYFIELD_SLOTS[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let playfield_h = ((playfield::PLAYFIELD_SLOTS[1] - 2) as f32
            * (BLOCK_SIZE + BLOCK_PADDING))
            + BLOCK_PADDING;
        let staging_w = playfield_w;
        let staging_h = (2. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        let hold_w = preview_w;
        let hold_h = staging_h;

        let playfield_x = VIEW_WH[0] / 2. - playfield_w / 2.;
        let playfield_y = VIEW_WH[1] / 2. - playfield_h / 2. + staging_h / 2. + 1.;
        let staging_x = playfield_x;
        let staging_y = playfield_y - staging_h - STAGING_PADDING;
        let preview_x = playfield_x + playfield_w + 10.;
        let preview_y = playfield_y;
        let hold_x = playfield_x - preview_w - 10.;
        let hold_y = playfield_y;

        Self {
            view_w: VIEW_WH[0],
            view_h: VIEW_WH[1],
            view_rect: Rect::new(0., 0., VIEW_WH[0], VIEW_WH[1]),
            playfield_rect: Rect::new(playfield_x, playfield_y, playfield_w, playfield_h),
            staging_rect: Rect::new(staging_x, staging_y, staging_w, staging_h),
            preview_rect: Rect::new(preview_x, preview_y, preview_w, preview_h),
            hold_rect: Rect::new(hold_x, hold_y, hold_w, hold_h),
            score_label_pos: Vec2::new(
                playfield_x + playfield_w + 30.,
                playfield_y + playfield_h - 30.,
            ),
            level_label_pos: Vec2::new(playfield_x - 180., playfield_y + playfield_h - 30.),
            title_pos: Vec2::new(playfield_x - 280., playfield_y - 50.),
            level_pos: Vec2::new(playfield_x - 60., playfield_y + playfield_h - 30.),
            score_pos: Vec2::new(
                playfield_x + playfield_w + 150.,
                playfield_y + playfield_h - 30.,
            ),
        }
    }
}

pub fn draw(game: &RustrisGame, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    match game.state {
        game::GameState::Menu => {
            draw_playing_backgound(ctx, canvas)?;
            draw_menu(ctx, canvas)?;
            draw_help_text(ctx, canvas)?;
        }
        game::GameState::Playing => {
            draw_playing_backgound(ctx, canvas)?;
            // draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino,ctx, canvas);
            // draw_playing_overlay(game.level, game.score,ctx, canvas);
        }
        game::GameState::Paused => {
            draw_playing_backgound(ctx, canvas)?;
            // draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino);
            // draw_playing_overlay(game.level, game.score);
            // draw_paused();
            // draw_help_text();
        }
        game::GameState::GameOver => {
            draw_playing_backgound(ctx, canvas)?;
            // draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino);
            // draw_playing_overlay(game.level, game.score);
            // draw_gameover()
        }
    }
    Ok(())
}

pub fn draw_playing_backgound(ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    let staging_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.staging_rect,
        STAGING_BACKGROUND_COLOR,
    )?;
    canvas.draw(&staging_rect, graphics::DrawParam::default());

    //     draw_rectangle(
    //         VIEW_SETTINGS.staging_rect.x,
    //         VIEW_SETTINGS.staging_rect.y,
    //         VIEW_SETTINGS.staging_rect.w,
    //         VIEW_SETTINGS.staging_rect.h,
    //         STAGING_BACKGROUND_COLOR,
    //     );

    let playfield_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.playfield_rect,
        PLAYFIELD_BACKGROUND_COLOR,
    )?;
    canvas.draw(&playfield_rect, graphics::DrawParam::default());

    //     draw_rectangle(
    //         VIEW_SETTINGS.playfield_rect.x,
    //         VIEW_SETTINGS.playfield_rect.y,
    //         VIEW_SETTINGS.playfield_rect.w,
    //         VIEW_SETTINGS.playfield_rect.h,
    //         PLAYFIELD_BACKGROUND_COLOR,
    //     );

    let preview_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.preview_rect,
        PREVIEW_BACKGROUND_COLOR,
    )?;
    canvas.draw(&preview_rect, graphics::DrawParam::default());

    //     draw_rectangle(
    //         VIEW_SETTINGS.preview_rect.x,
    //         VIEW_SETTINGS.preview_rect.y,
    //         VIEW_SETTINGS.preview_rect.w,
    //         VIEW_SETTINGS.preview_rect.h,
    //         PREVIEW_BACKGROUND_COLOR,
    //     );

    let hold_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.hold_rect,
        PREVIEW_BACKGROUND_COLOR,
    )?;
    canvas.draw(&hold_rect, graphics::DrawParam::default());

    //     draw_rectangle(
    //         VIEW_SETTINGS.hold_rect.x,
    //         VIEW_SETTINGS.hold_rect.y,
    //         VIEW_SETTINGS.hold_rect.w,
    //         VIEW_SETTINGS.hold_rect.h,
    //         HOLD_BACKGROUND_COLOR,
    //     );
    Ok(())
}

pub fn draw_playing(
    playfield: &RustrisPlayfield,
    next_rustomino: &Option<Rustomino>,
    held_rustomino: &Option<Rustomino>,
    ctx: &mut Context,
    canvas: &mut Canvas,
) -> GameResult {
    //     for (y, slots_x) in playfield.slots.iter().enumerate() {
    //         for (x, slot) in slots_x.iter().enumerate() {
    //             match slot {
    //                 SlotState::Locked(rtype) | SlotState::Occupied(rtype) => {
    //                     // draw the block
    //                     let rect = playfield_block_rect([x as i32, y as i32]);
    //                     draw_rectangle(rect.x, rect.y, rect.w, rect.h, rtype.color());
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }

    //     if let Some(next) = next_rustomino {
    //         for slot in next.blocks {
    //             // display the preview
    //             // draw the block
    //             let rect = next_block_rect([slot[0], slot[1]]);
    //             draw_rectangle(rect.x, rect.y, rect.w, rect.h, next.rtype.color());
    //         }
    //     }

    //     if let Some(held) = held_rustomino {
    //         for slot in held.blocks {
    //             // display the preview
    //             // draw the block
    //             let rect = hold_block_rect([slot[0], slot[1]]);
    //             draw_rectangle(rect.x, rect.y, rect.w, rect.h, held.rtype.color());
    //         }
    //     }

    //     if let Some(ghost) = &playfield.ghost_rustomino {
    //         for block in ghost.playfield_slots() {
    //             // draw the block
    //             let rect = playfield_block_rect([block[0], block[1]]);
    //             draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 4., GHOST_COLOR);
    //         }
    //     }
    Ok(())
}

pub fn draw_playing_overlay(
    game_level: usize,
    score: usize,
    ctx: &mut Context,
    canvas: &mut Canvas,
) -> GameResult {
    //     draw_text_ex(
    //         "Rustris",
    //         VIEW_SETTINGS.title_pos.x as f32,
    //         VIEW_SETTINGS.title_pos.y as f32,
    //         *text_params,
    //     );

    //     draw_text_ex(
    //         "Level:",
    //         VIEW_SETTINGS.level_label_pos.x as f32,
    //         VIEW_SETTINGS.level_label_pos.y as f32,
    //         *text_params,
    //     );

    //     draw_text_ex(
    //         &game_level.to_string(),
    //         VIEW_SETTINGS.level_pos.x as f32,
    //         VIEW_SETTINGS.level_pos.y as f32,
    //         *text_params,
    //     );

    //     draw_text_ex(
    //         "Score:",
    //         VIEW_SETTINGS.score_label_pos.x as f32,
    //         VIEW_SETTINGS.score_label_pos.y as f32,
    //         *text_params,
    //     );

    //     draw_text_ex(
    //         &score.to_string(),
    //         VIEW_SETTINGS.score_pos.x as f32,
    //         VIEW_SETTINGS.score_pos.y as f32,
    //         *text_params,
    //     );
    Ok(())
}

pub fn draw_paused(ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    //     draw_rectangle(
    //         0.,
    //         0.,
    //         VIEW_SETTINGS.view_w as f32,
    //         VIEW_SETTINGS.view_h as f32,
    //         PAUSED_OVERLAY_COLOR,
    //     );
    //     draw_text_ex(
    //         "Paused",
    //         (VIEW_SETTINGS.view_w / 2 - 75) as f32,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         *text_params,
    //     );
    Ok(())
}

pub fn draw_menu(ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    let menu_overlay = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.view_rect,
        PAUSED_OVERLAY_COLOR,
    )?;
    canvas.draw(&menu_overlay, graphics::DrawParam::default());

    //     draw_rectangle(
    //         0.,
    //         0.,
    //         VIEW_SETTINGS.view_w as f32,
    //         VIEW_SETTINGS.view_h as f32,
    //         PAUSED_OVERLAY_COLOR,
    //     );

    //     draw_text_ex(
    //         "Welcome to",
    //         (VIEW_SETTINGS.view_w / 2 - 230) as f32,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         *text_params,
    //     );
    //     draw_text_ex(
    //         "R",
    //         560.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::I.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "u",
    //         588.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::O.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "s",
    //         612.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::T.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "t",
    //         637.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::L.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "r",
    //         662.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::S.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "i",
    //         686.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::J.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "s",
    //         700.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         TextParams {
    //             font: text_params.font,
    //             font_size: 30,
    //             color: RustominoType::Z.color(),
    //             ..Default::default()
    //         },
    //     );
    //     draw_text_ex(
    //         "!",
    //         725.,
    //         (VIEW_SETTINGS.view_h / 2 - 90) as f32,
    //         *text_params,
    //     );

    //     draw_text_ex(
    //         "Press Enter To Start",
    //         (VIEW_SETTINGS.view_w / 2 - 253) as f32,
    //         (VIEW_SETTINGS.view_h / 2 - 30) as f32,
    //         *text_params,
    //     );
    Ok(())
}

pub fn draw_gameover(ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    let gameover_overlay = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        VIEW_SETTINGS.view_rect,
        PAUSED_OVERLAY_COLOR,
    )?;
    canvas.draw(&gameover_overlay, graphics::DrawParam::default());

    //     draw_rectangle(
    //         0.,
    //         0.,
    //         VIEW_SETTINGS.view_w as f32,
    //         VIEW_SETTINGS.view_h as f32,
    //         PAUSED_OVERLAY_COLOR,
    //     );
    //     draw_text_ex(
    //         "Game Over!",
    //         (VIEW_SETTINGS.view_w / 2 - 122) as f32,
    //         (VIEW_SETTINGS.view_h / 2 - 30) as f32,
    //         *text_params,
    //     );
    //     draw_text_ex(
    //         "Press Enter To Play Again",
    //         (VIEW_SETTINGS.view_w / 2 - 310) as f32,
    //         (VIEW_SETTINGS.view_h / 2 + 30) as f32,
    //         *text_params,
    //     );

    Ok(())
}

pub fn draw_help_text(ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
    let help_overlay = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        graphics::Rect::new(285., 410., 445., 305.),
        PAUSED_OVERLAY_COLOR,
    )?;
    canvas.draw(&help_overlay, graphics::DrawParam::default());
    //     draw_rectangle(285., 410., 445., 305., CONTROLS_BACKGROUND_COLOR);

    //     draw_text_ex(
    //         "Controls:",
    //         305.,
    //         (VIEW_SETTINGS.view_h / 2 + 65) as f32,
    //         *font_30pt,
    //     );
    //     draw_text_ex(
    //         "Move Left: Left, A",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 98) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Move Right: Right, D",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 128) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Rotate CW: Up, W",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 157) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Rotate CCW: LCtrl, Z",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 187) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Soft Drop: Down, S",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 217) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Hard Drop: Space",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 247) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Hold: LShift, C",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 277) as f32,
    //         *font_20pt,
    //     );

    //     draw_text_ex(
    //         "Adjust Music Volume: + -",
    //         315.,
    //         (VIEW_SETTINGS.view_h / 2 + 307) as f32,
    //         *font_20pt,
    //     );
    //     // Hold: LShift, C Music Volume: + -

    Ok(())
}

fn next_block_rect(block: [i32; 2]) -> Rect {
    // block[x,y] absolute units
    let x = VIEW_SETTINGS.preview_rect.x
        + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        + 1.0;
    // get bottom left of playfield_rect
    let y = VIEW_SETTINGS.preview_rect.y + VIEW_SETTINGS.preview_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn hold_block_rect(block: [i32; 2]) -> Rect {
    // block[x,y] absolute units
    let x =
        VIEW_SETTINGS.hold_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32) + 1.0;
    // get bottom left of playfield_rect
    let y = VIEW_SETTINGS.hold_rect.y + VIEW_SETTINGS.hold_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn playfield_block_rect(block: [i32; 2]) -> Rect {
    // block[x,y] absolute units
    let x = VIEW_SETTINGS.staging_rect.x
        + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        + 1.0;
    // get bottom left of playfield_rect
    let y = VIEW_SETTINGS.playfield_rect.y + VIEW_SETTINGS.playfield_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        - 1.0;

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

// #[derive(Debug, Clone, Copy)]
// pub struct Rect<T> {
//     pub x: T,
//     pub y: T,
//     pub w: T,
//     pub h: T,
// }

// impl<T> Rect<T> {
//     pub const fn new(x: T, y: T, w: T, h: T) -> Self {
//         Rect { x, y, w, h }
//     }
// }

// impl<T> From<[T; 4]> for Rect<T>
// where
//     T: Copy,
// {
//     fn from(value: [T; 4]) -> Self {
//         Rect {
//             x: value[0],
//             y: value[1],
//             w: value[2],
//             h: value[3],
//         }
//     }
// }

// impl<T> From<Rect<T>> for [T; 4] {
//     fn from(value: Rect<T>) -> Self {
//         [value.x, value.y, value.w, value.h]
//     }
// }
