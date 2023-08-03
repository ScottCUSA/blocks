use crate::game::{self, RustrisGame};
use crate::playfield::{self, RustrisPlayfield, SlotState};
use crate::rustomino::{Rustomino, RustominoType};
use macroquad::prelude::*;

const BLOCK_SIZE: i32 = 30;
const BLOCK_PADDING: i32 = 1;
const STAGING_PADDING: i32 = 2;

pub const BACKGROUND_COLOR: Color = Color::new(0.0, 0.29, 0.38, 1.0);
const STAGING_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.2);
const GHOST_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
const PAUSED_OVERLAY_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.6);
const VIEW_SETTINGS: ViewSettings = ViewSettings::new(VIEW_WH);
const CONTROLS_BACKGROUND_COLOR: Color = Color::new(0.34, 0.09, 0.12, 0.8);
pub const VIEW_WH: [i32; 2] = [1024, 768];

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: VIEW_WH[0],
        window_height: VIEW_WH[1],
        window_resizable: true,
        ..Default::default()
    }
}

pub struct ViewSettings {
    pub view_w: i32,
    pub view_h: i32,
    pub playfield_rect: Rect<f32>,
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
        let playfield_w =
            (playfield::PLAYFIELD_SLOTS[0] as i32 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let playfield_h = ((playfield::PLAYFIELD_SLOTS[1] as i32 - 2)
            * (BLOCK_SIZE + BLOCK_PADDING))
            + BLOCK_PADDING;
        let staging_w = playfield_w;
        let staging_h = (2 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_w = (4 * (BLOCK_SIZE + BLOCK_PADDING)) + BLOCK_PADDING;
        let preview_h = staging_h;
        let hold_w = preview_w;
        let hold_h = staging_h;

        let playfield_x = view_dimensions[0] / 2 - playfield_w / 2;
        let playfield_y = view_dimensions[1] / 2 - playfield_h / 2 + staging_h / 2 + 1;
        let staging_x = playfield_x;
        let staging_y = playfield_y - staging_h - STAGING_PADDING;
        let preview_x = playfield_x + playfield_w + 10;
        let preview_y = playfield_y;
        let hold_x = playfield_x - preview_w - 10;
        let hold_y = playfield_y;

        Self {
            view_w: view_dimensions[0],
            view_h: view_dimensions[1],
            playfield_rect: Rect::new(
                playfield_x as f32,
                playfield_y as f32,
                playfield_w as f32,
                playfield_h as f32,
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
            score_label_pos: ivec2(
                playfield_x + playfield_w + 30,
                playfield_y + playfield_h - 30,
            ),
            level_label_pos: ivec2(playfield_x - 180, playfield_y + playfield_h - 30),
            title_pos: ivec2(playfield_x - 280, playfield_y - 50),
            level_pos: ivec2(playfield_x - 60, playfield_y + playfield_h - 30),
            score_pos: ivec2(
                playfield_x + playfield_w + 150,
                playfield_y + playfield_h - 30,
            ),
        }
    }
}

pub fn draw(game: &RustrisGame, font: Option<&Font>) {
    match game.state {
        game::GameState::Menu => {
            draw_playing_backgound();
            draw_menu(font);
            draw_help_text(font);
        }
        game::GameState::Playing => {
            draw_playing_backgound();
            draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino);
            draw_playing_overlay(font, game.level, game.score);
        }
        game::GameState::Paused => {
            draw_playing_backgound();
            draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino);
            draw_playing_overlay(font, game.level, game.score);
            draw_paused(font);
            draw_help_text(font);
        }
        game::GameState::GameOver => {
            draw_playing_backgound();
            draw_playing(&game.playfield, &game.next_rustomino, &game.held_rustomino);
            draw_playing_overlay(font, game.level, game.score);
            draw_gameover(font)
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
        VIEW_SETTINGS.playfield_rect.x,
        VIEW_SETTINGS.playfield_rect.y,
        VIEW_SETTINGS.playfield_rect.w,
        VIEW_SETTINGS.playfield_rect.h,
        PLAYFIELD_BACKGROUND_COLOR,
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
    playfield: &RustrisPlayfield,
    next_rustomino: &Option<Rustomino>,
    held_rustomino: &Option<Rustomino>,
) {
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        for (x, slot) in slots_x.iter().enumerate() {
            match slot {
                SlotState::Locked(rtype) | SlotState::Occupied(rtype) => {
                    // draw the block
                    let rect = playfield_block_rect([x as i32, y as i32]);
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
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, next.rtype.color());
        }
    }

    if let Some(held) = held_rustomino {
        for slot in held.blocks {
            // display the preview
            // draw the block
            let rect = hold_block_rect([slot[0], slot[1]]);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, held.rtype.color());
        }
    }

    if let Some(ghost) = &playfield.ghost_rustomino {
        for block in ghost.playfield_slots() {
            // draw the block
            let rect = playfield_block_rect([block[0], block[1]]);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 4., GHOST_COLOR);
        }
    }
}

pub fn draw_playing_overlay(font: Option<&Font>, game_level: usize, score: usize) {
    draw_text_ex(
        "Rustris",
        VIEW_SETTINGS.title_pos.x as f32,
        VIEW_SETTINGS.title_pos.y as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Level:",
        VIEW_SETTINGS.level_label_pos.x as f32,
        VIEW_SETTINGS.level_label_pos.y as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        &game_level.to_string(),
        VIEW_SETTINGS.level_pos.x as f32,
        VIEW_SETTINGS.level_pos.y as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Score:",
        VIEW_SETTINGS.score_label_pos.x as f32,
        VIEW_SETTINGS.score_label_pos.y as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        &score.to_string(),
        VIEW_SETTINGS.score_pos.x as f32,
        VIEW_SETTINGS.score_pos.y as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
}

pub fn draw_paused(font: Option<&Font>) {
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
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
}

pub fn draw_menu(font: Option<&Font>) {
    draw_rectangle(
        0.,
        0.,
        VIEW_SETTINGS.view_w as f32,
        VIEW_SETTINGS.view_h as f32,
        PAUSED_OVERLAY_COLOR,
    );
    draw_text_ex(
        "Welcome to",
        (VIEW_SETTINGS.view_w / 2 - 230) as f32,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
    draw_text_ex(
        "R",
        560.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::I.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "u",
        588.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::O.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "s",
        612.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::T.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "t",
        637.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::L.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "r",
        662.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::S.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "i",
        686.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::J.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "s",
        700.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 30,
            color: RustominoType::Z.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        "!",
        725.,
        (VIEW_SETTINGS.view_h / 2 - 90) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Press Enter To Start",
        (VIEW_SETTINGS.view_w / 2 - 253) as f32,
        (VIEW_SETTINGS.view_h / 2 - 30) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
}

pub fn draw_gameover(font: Option<&Font>) {
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
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
    draw_text_ex(
        "Press Enter To Play Again",
        (VIEW_SETTINGS.view_w / 2 - 310) as f32,
        (VIEW_SETTINGS.view_h / 2 + 30) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
}

///
pub fn draw_help_text(font: Option<&Font>) {
    draw_rectangle(285., 410., 445., 305., CONTROLS_BACKGROUND_COLOR);

    draw_text_ex(
        "Controls:",
        305.,
        (VIEW_SETTINGS.view_h / 2 + 65) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
    draw_text_ex(
        "Move Left: Left, A",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 98) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Move Right: Right, D",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 128) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Rotate CW: Up, W",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 157) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Rotate CCW: LCtrl, Z",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 187) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Soft Drop: Down, S",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 217) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Hard Drop: Space",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 247) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Hold: LShift, C",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 277) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Adjust Music Volume: + -",
        315.,
        (VIEW_SETTINGS.view_h / 2 + 307) as f32,
        TextParams {
            font,
            font_size: 20,
            ..Default::default()
        },
    );
    // Hold: LShift, C Music Volume: + -
}

fn next_block_rect(block: [i32; 2]) -> Rect<f32> {
    // block[x,y] absolute units
    let x = VIEW_SETTINGS.preview_rect.x
        + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32)
        + 1.0;
    // get bottom left of playfield_rect
    let y = VIEW_SETTINGS.preview_rect.y + VIEW_SETTINGS.preview_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn hold_block_rect(block: [i32; 2]) -> Rect<f32> {
    // block[x,y] absolute units
    let x =
        VIEW_SETTINGS.hold_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32) + 1.0;
    // get bottom left of playfield_rect
    let y = VIEW_SETTINGS.hold_rect.y + VIEW_SETTINGS.hold_rect.h
        - (block[1] as f32 * (BLOCK_SIZE + BLOCK_PADDING) as f32);

    Rect::new(x, y, BLOCK_SIZE as f32, BLOCK_SIZE as f32)
}

fn playfield_block_rect(block: [i32; 2]) -> Rect<f32> {
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
