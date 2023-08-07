use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Color, DrawMode, Rect, StrokeOptions};
use ggez::{Context, GameResult};

use crate::menus::{self, Menu};
use crate::playfield::{self, RustrisPlayfield, SlotState};
use crate::rustomino::Rustomino;
use crate::util;

const BLOCK_SIZE: f32 = 30.;
const BLOCK_PADDING: f32 = 1.;
const STAGING_PADDING: f32 = 2.;

pub const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
pub const VIEW_WIDTH: f32 = 1024.0;
pub const VIEW_HEIGHT: f32 = 768.;
const UI_FONT_SIZE: f32 = 24.0;
pub const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.2);
const GHOST_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
// const CONTROLS_BACKGROUND_COLOR: Color = Color::new(0.34, 0.09, 0.12, 0.8);

#[derive(Debug)]
pub struct ViewSettings {
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
    pub fn new(drawable_width: f32, drawable_height: f32) -> Self {
        // calculate the playfield dimensions based on block size, padding and playfield slots
        let playfield_w =
            (playfield::PLAYFIELD_SLOTS[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let playfield_h = ((playfield::PLAYFIELD_SLOTS[1] - 2) as f32
            * (BLOCK_SIZE + BLOCK_PADDING))
            + BLOCK_PADDING;

        // calculate the dimentions of the staging area
        let staging_w = playfield_w;
        let staging_h = (2. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        // calculate the dimensions of the preview area
        let preview_w = (4. * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        // calculate the dimensions of the hold area
        let hold_w = preview_w;
        let hold_h = staging_h;

        // center playfield
        let playfield_x = drawable_width / 2.0 - playfield_w / 2.0;
        let playfield_y = drawable_height / 2.0 - playfield_h / 2.0 + staging_h / 2.0 + 1.0;
        // center staging area above playfield
        let staging_x = playfield_x;
        let staging_y = playfield_y - staging_h - STAGING_PADDING;
        // center preview area to the right of playfield
        let preview_x = playfield_x + playfield_w + 10.0;
        let preview_y = playfield_y;
        // center hold area to the left of playfield
        let hold_x = playfield_x - preview_w - 10.0;
        let hold_y = playfield_y;

        Self {
            view_rect: Rect::new(0., 0., drawable_width, drawable_height),
            playfield_rect: Rect::new(playfield_x, playfield_y, playfield_w, playfield_h),
            staging_rect: Rect::new(staging_x, staging_y, staging_w, staging_h),
            preview_rect: Rect::new(preview_x, preview_y, preview_w, preview_h),
            hold_rect: Rect::new(hold_x, hold_y, hold_w, hold_h),
            score_label_pos: Vec2::new(
                playfield_x + playfield_w + 30.0,
                playfield_y + playfield_h - 30.0,
            ),
            level_label_pos: Vec2::new(playfield_x - 180.0, playfield_y + playfield_h - 30.0),
            title_pos: Vec2::new(playfield_x - 280.0, playfield_y - 50.0),
            level_pos: Vec2::new(playfield_x - 60.0, playfield_y + playfield_h - 30.0),
            score_pos: Vec2::new(
                playfield_x + playfield_w + 150.0,
                playfield_y + playfield_h - 30.0,
            ),
        }
    }
}

pub fn draw_playing_backgound(
    ctx: &mut Context,
    canvas: &mut Canvas,
    view_settings: &ViewSettings,
) -> GameResult {
    // draw the staging background
    let staging_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.staging_rect,
        STAGING_BACKGROUND_COLOR,
    )?;
    canvas.draw(&staging_rect, graphics::DrawParam::default());

    // draw the playfield background
    let playfield_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.playfield_rect,
        PLAYFIELD_BACKGROUND_COLOR,
    )?;
    canvas.draw(&playfield_rect, graphics::DrawParam::default());

    // draw the preview background
    let preview_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.preview_rect,
        PREVIEW_BACKGROUND_COLOR,
    )?;
    canvas.draw(&preview_rect, graphics::DrawParam::default());

    // draw the hold background
    let hold_rect = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.hold_rect,
        HOLD_BACKGROUND_COLOR,
    )?;
    canvas.draw(&hold_rect, graphics::DrawParam::default());

    Ok(())
}

fn draw_playfield(
    ctx: &mut Context,
    canvas: &mut Canvas,
    playfield: &RustrisPlayfield,
    staging_rect: &Rect,
    playfield_rect: &Rect,
    game_over: bool,
) -> GameResult {
    // create a mesh we'll reuse for each block
    let block_mesh = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
    )?;

    // draw the playfield
    let draw_param = graphics::DrawParam::default();
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        for (x, slot) in slots_x.iter().enumerate() {
            match slot {
                SlotState::Locked(rtype) | SlotState::Occupied(rtype) => {
                    // draw the block
                    let rect =
                        playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                    let color = if game_over {
                        util::rgb_to_grayscale(rtype.color())
                    } else {
                        rtype.color()
                    };
                    canvas.draw(&block_mesh, draw_param.dest_rect(rect).color(color));
                }
                _ => {}
            }
        }
    }

    let ghost_mesh = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::Stroke(StrokeOptions::default().with_line_width(0.1)),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        GHOST_COLOR,
    )?;

    if let Some(ghost) = &playfield.ghost_rustomino {
        for block in ghost.playfield_slots() {
            // draw the block
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            canvas.draw(&ghost_mesh, draw_param.dest_rect(rect));
        }
    }

    Ok(())
}

fn draw_hold(
    ctx: &mut Context,
    canvas: &mut Canvas,
    hold_rustomino: &Option<Rustomino>,
    hold_rect: &Rect,
    game_over: bool,
) -> GameResult {
    // create a mesh we'll reuse for each block
    let mesh = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
    )?;

    let draw_param = graphics::DrawParam::new();
    if let Some(next) = hold_rustomino {
        for block in next.blocks {
            let rect = hold_block_rect([block[0], block[1]], hold_rect);
            let color = if game_over {
                util::rgb_to_grayscale(next.rtype.color())
            } else {
                next.rtype.color()
            };
            canvas.draw(&mesh, draw_param.dest_rect(rect).color(color));
        }
    }
    Ok(())
}

fn draw_next(
    ctx: &mut Context,
    canvas: &mut Canvas,
    next_rustomino: &Option<Rustomino>,
    next_rect: &Rect,
    game_over: bool,
) -> GameResult {
    // create a mesh we'll reuse for each block
    let mesh = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
    )?;

    let draw_param = graphics::DrawParam::new();
    if let Some(next) = next_rustomino {
        for block in next.blocks {
            let rect = next_block_rect([block[0], block[1]], next_rect);
            let color = if game_over {
                util::rgb_to_grayscale(next.rtype.color())
            } else {
                next.rtype.color()
            };
            canvas.draw(&mesh, draw_param.dest_rect(rect).color(color));
        }
    }
    Ok(())
}

pub fn draw_playing(
    ctx: &mut Context,
    canvas: &mut Canvas,
    playfield: &RustrisPlayfield,
    next_rustomino: &Option<Rustomino>,
    hold_rustomino: &Option<Rustomino>,
    view_settings: &ViewSettings,
    game_over: bool,
) -> GameResult {
    draw_playing_backgound(ctx, canvas, view_settings)?;
    draw_playfield(
        ctx,
        canvas,
        playfield,
        &view_settings.staging_rect,
        &view_settings.playfield_rect,
        game_over,
    )?;
    draw_hold(
        ctx,
        canvas,
        hold_rustomino,
        &view_settings.hold_rect,
        game_over,
    )?;
    draw_next(
        ctx,
        canvas,
        next_rustomino,
        &view_settings.preview_rect,
        game_over,
    )?;

    Ok(())
}

pub fn draw_playing_text(
    _ctx: &mut Context,
    canvas: &mut Canvas,
    level: usize,
    score: usize,
    view_settings: &ViewSettings,
) -> GameResult {
    let mut title_text = graphics::Text::new("Rustris");
    let mut level_text = graphics::Text::new("Level:");
    let mut score_text = graphics::Text::new("Score:");

    let text_param = graphics::DrawParam::default();

    canvas.draw(
        title_text
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(UI_FONT_SIZE)),
        text_param
            .dest([view_settings.title_pos.x, view_settings.title_pos.y])
            .color(Color::new(1., 1., 1., 1.)),
    );

    canvas.draw(
        level_text
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(UI_FONT_SIZE)),
        text_param
            .dest([
                view_settings.level_label_pos.x,
                view_settings.level_label_pos.y,
            ])
            .color(Color::new(1., 1., 1., 1.)),
    );

    canvas.draw(
        score_text
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(UI_FONT_SIZE)),
        text_param
            .dest([
                view_settings.score_label_pos.x,
                view_settings.score_label_pos.y,
            ])
            .color(Color::new(1., 1., 1., 1.)),
    );

    canvas.draw(
        graphics::Text::new(level.to_string())
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(UI_FONT_SIZE)),
        text_param
            .dest([view_settings.level_pos.x, view_settings.level_pos.y])
            .color(Color::new(1., 1., 1., 1.)),
    );
    canvas.draw(
        graphics::Text::new(score.to_string())
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(UI_FONT_SIZE)),
        text_param
            .dest([view_settings.score_pos.x, view_settings.score_pos.y])
            .color(Color::new(1., 1., 1., 1.)),
    );

    Ok(())
}

pub fn draw_menu_background(
    ctx: &mut Context,
    canvas: &mut Canvas,
    view_settings: &ViewSettings,
) -> GameResult {
    // for now this is just a static transparent overlay
    let menu_overlay = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.view_rect,
        PAUSED_OVERLAY_COLOR,
    )?;
    canvas.draw(&menu_overlay, graphics::DrawParam::default());
    Ok(())
}

fn draw_menu_text<T: Menu>(
    ctx: &mut Context,
    canvas: &mut Canvas,
    menu_state: &T,
    view_settings: &ViewSettings,
    title: &str,
) -> GameResult {
    let time = ctx.time.time_since_start().as_secs_f32();

    let slow_wobble = util::slow_wobble(time);
    let fast_wobble = util::fast_wobble(time);

    let title_scale = graphics::PxScale::from(100.0);
    let font_scale = graphics::PxScale::from(50.0);

    let mut title = graphics::Text::new(title);

    let scaled_title = title.set_font("04b30").set_scale(title_scale);

    let title_glyph_pos = scaled_title.glyph_positions(ctx)?;
    let title_width = title_glyph_pos.last().unwrap().x - title_glyph_pos.first().unwrap().x
        + title_scale.x / 2.0;
    let title_x = view_settings.view_rect.w / 2.0 - title_width / 2.0;
    let title_y = view_settings.view_rect.h / 4.0 - title_scale.y / 2.0 + (slow_wobble * 10.0);

    let title_draw_param = graphics::DrawParam::default()
        .dest([title_x, title_y])
        .color(Color::new(1., 1., 1., 1.));

    // draw rustris title
    canvas.draw(scaled_title, title_draw_param);

    for (i, item) in menu_state.items().iter().enumerate() {
        let mut item = item.clone();
        let scaled_text = item.set_font("04b30").set_scale(font_scale);
        let glyph_pos = scaled_text.glyph_positions(ctx)?;
        let item_width =
            glyph_pos.last().unwrap().x - glyph_pos.first().unwrap().x + font_scale.x / 2.0;
        let menu_item_height = font_scale.y;
        let x_pos = if menu_state.selected() == i {
            view_settings.view_rect.w / 2.0 - item_width / 2.0 + fast_wobble * 5.0
        } else {
            view_settings.view_rect.w / 2.0 - item_width / 2.0
        };
        let menu_item_draw_param = graphics::DrawParam::default()
            .dest([
                x_pos,
                view_settings.view_rect.h / 1.9 + (menu_item_height * (i as f32)),
            ])
            .color(Color::new(1., 1., 1., 1.));
        canvas.draw(scaled_text, menu_item_draw_param);
    }

    Ok(())
}

pub fn draw_menu(
    ctx: &mut Context,
    canvas: &mut Canvas,
    menu_state: &menus::MenuState,
    view_settings: &ViewSettings,
) -> GameResult {
    // draw the menu background
    draw_menu_background(ctx, canvas, view_settings)?;
    draw_menu_text(ctx, canvas, menu_state, view_settings, "Rustris!")?;
    // draw_main_menu_text(ctx, canvas, menu_state, view_settings)?;
    Ok(())
}

pub fn draw_gameover(ctx: &mut Context, canvas: &mut Canvas, view_rect: &Rect) -> GameResult {
    let gameover_overlay =
        graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), *view_rect, PAUSED_OVERLAY_COLOR)?;
    canvas.draw(&gameover_overlay, graphics::DrawParam::default());

    let slow_wobble = util::slow_wobble(ctx.time.time_since_start().as_secs_f32());

    let mut scaled_text = graphics::Text::new("Game Over!");
    let scaled_text = scaled_text
        .set_font("04b30")
        .set_scale(graphics::PxScale::from(50.0));
    let glyph_pos = scaled_text.glyph_positions(ctx)?;
    let text_width = glyph_pos.last().unwrap().x - glyph_pos.first().unwrap().x + 25.0;
    canvas.draw(
        graphics::Text::new("Game Over!")
            .set_font("04b30")
            .set_scale(graphics::PxScale::from(50.0)),
        graphics::DrawParam::default()
            .dest([
                view_rect.w / 2.0 - text_width / 2.0,
                view_rect.h / 2.0 - 25.0 - slow_wobble * 10.0,
            ])
            .color(Color::new(1., 1., 1., 1.)),
    );
    Ok(())
}

// pub fn draw_options(
//     ctx: &mut Context,
//     canvas: &mut Canvas,
//     view_rect: &Rect,
// ) -> GameResult {
//     let help_overlay = graphics::Mesh::new_rectangle(
//         ctx,
//         DrawMode::fill(),
//         graphics::Rect::new(285., 410., 445., 305.),
//         PAUSED_OVERLAY_COLOR,
//     )?;
//     canvas.draw(&help_overlay, graphics::DrawParam::default());
//     //     draw_rectangle(285., 410., 445., 305., CONTROLS_BACKGROUND_COLOR);

//     //     draw_text_ex(
//     //         "Controls:",
//     //         305.,
//     //         (view_rect.h / 2 + 65) as f32,
//     //         *font_30pt,
//     //     );
//     //     draw_text_ex(
//     //         "Move Left: Left, A",
//     //         315.,
//     //         (view_rect.h / 2 + 98) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Move Right: Right, D",
//     //         315.,
//     //         (view_rect.h / 2 + 128) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Rotate CW: Up, W",
//     //         315.,
//     //         (view_rect.h / 2 + 157) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Rotate CCW: LCtrl, Z",
//     //         315.,
//     //         (view_rect.h / 2 + 187) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Soft Drop: Down, S",
//     //         315.,
//     //         (view_rect.h / 2 + 217) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Hard Drop: Space",
//     //         315.,
//     //         (view_rect.h / 2 + 247) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Hold: LShift, C",
//     //         315.,
//     //         (view_rect.h / 2 + 277) as f32,
//     //         *font_20pt,
//     //     );

//     //     draw_text_ex(
//     //         "Adjust Music Volume: + -",
//     //         315.,
//     //         (view_rect.h / 2 + 307) as f32,
//     //         *font_20pt,
//     //     );
//     //     // Hold: LShift, C Music Volume: + -

//     Ok(())
// }

pub fn draw_paused(
    ctx: &mut Context,
    canvas: &mut Canvas,
    paused_state: &menus::PausedState,
    view_settings: &ViewSettings,
) -> GameResult {
    // draw the menu background
    draw_paused_background(ctx, canvas, view_settings)?;
    draw_menu_text(ctx, canvas, paused_state, view_settings, "Paused!")?;
    Ok(())
}

pub fn draw_paused_background(
    ctx: &mut Context,
    canvas: &mut Canvas,
    view_settings: &ViewSettings,
) -> GameResult {
    // for now this is just a static transparent overlay
    let menu_overlay = graphics::Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        view_settings.view_rect,
        PAUSED_OVERLAY_COLOR,
    )?;
    canvas.draw(&menu_overlay, graphics::DrawParam::default());
    Ok(())
}

fn next_block_rect(block: [i32; 2], preview_rect: &Rect) -> Rect {
    // block[x,y] absolute units
    let x = preview_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = preview_rect.y + preview_rect.h - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING));

    Rect::new(x, y, BLOCK_SIZE, BLOCK_SIZE)
}

fn hold_block_rect(block: [i32; 2], hold_rect: &Rect) -> Rect {
    // block[x,y] absolute units
    let x = hold_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = hold_rect.y + hold_rect.h - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING));

    Rect::new(x, y, BLOCK_SIZE, BLOCK_SIZE)
}

fn playfield_block_rect(block: [i32; 2], staging_rect: &Rect, playfield_rect: &Rect) -> Rect {
    // block[x,y] absolute units
    let x = staging_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = playfield_rect.y + playfield_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING))
        - 1.0;

    Rect::new(x, y, BLOCK_SIZE, BLOCK_SIZE)
}
