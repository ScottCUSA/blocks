use crate::gfx::Vec2;
use crate::gfx::{Color, Rect};
use crate::GameResult;

use crate::effects::GameOver;
use crate::rustomino::RustominoType;
use strum::IntoEnumIterator;

use super::*;
use crate::draw::*;

pub(super) const GAME_OVER_TITLE_SIZE: f32 = 56.0;
pub(super) const GAME_OVER_TITLE_ROW: f32 = 100.0;
pub(super) const GAME_OVER_STAT_ROW: f32 = 44.0;
pub(super) const GAME_OVER_STAT_SIZE: f32 = 28.0;
pub(super) const LETTER_DROP: f32 = 140.0; // px
pub(super) const LETTER_WAVE: f32 = 4.0; // px

/// game over panel, over the game
pub struct GameOverScreen<'a> {
    pub game_over: &'a GameOver,
    pub score: usize,
    pub level: usize,
}

impl Layer for GameOverScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self {
            game_over,
            score,
            level,
        } = *self;
        let meshes = frame.meshes;
        let view_rect = &frame.view.view_rect;
        draw_overlay_alpha(canvas, meshes, view_rect, game_over.overlay());
        let drop = game_over.panel();
        if drop <= 0.0 {
            return Ok(());
        }
        let time = game_over.time();
        let height = GAME_OVER_TITLE_ROW + 2.0 * GAME_OVER_STAT_ROW + PANEL_PADDING;
        let top = (view_rect.h - height) / 2.0 - (1.0 - ease_out_back(drop)) * view_rect.h;
        let panel = Rect::new(
            (view_rect.w - MENU_PANEL_WIDTH) / 2.0,
            top,
            MENU_PANEL_WIDTH,
            height,
        );
        draw_panel(canvas, meshes, panel);

        let title_center = Vec2::new(view_rect.w / 2.0, top + GAME_OVER_TITLE_ROW / 2.0 + 4.0);
        draw_dropping_title(canvas, game_over, "Game Over!", title_center)?;

        // final score and level count up
        let count = 1.0 - (1.0 - game_over.count()).powi(3);
        let stats = [
            ("Score", (score as f32 * count).round() as usize),
            ("Level", (level as f32 * count).round() as usize),
        ];
        let left = panel.x + 2.0 * PANEL_PADDING;
        let right = panel.x + panel.w - 2.0 * PANEL_PADDING;
        for (i, (label, value)) in stats.into_iter().enumerate() {
            let center_y = top + GAME_OVER_TITLE_ROW + (i as f32 + 0.5) * GAME_OVER_STAT_ROW;
            let label = ui_text(label, GAME_OVER_STAT_SIZE);
            let label_h = label.measure_ui().y;
            draw_shadowed(
                canvas,
                &label,
                Vec2::new(left, center_y - label_h / 2.0),
                LABEL_COLOR,
            );
            let value = ui_text(&value.to_string(), GAME_OVER_STAT_SIZE);
            let dims = value.measure_ui();
            let counting = game_over.count() > 0.0 && game_over.count() < 1.0;
            let color = if counting { ACCENT_COLOR } else { Color::WHITE };
            draw_shadowed(
                canvas,
                &value,
                Vec2::new(right - dims.x, center_y - dims.y / 2.0),
                color,
            );
        }

        if game_over.ready() {
            let blink = 0.65 + 0.35 * (time * 4.0).sin();
            let hint = ui_text("Press any key", SMALL_TEXT_SIZE);
            let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
            draw_text_centered(canvas, &hint, center, with_alpha(LABEL_COLOR, blink))?;
        }
        Ok(())
    }
}

/// piece colored letters that drop in one by one and bounce, then wave
pub(super) fn draw_dropping_title(
    canvas: &mut Painter,
    game_over: &GameOver,
    title: &str,
    center: Vec2,
) -> GameResult {
    let size = GAME_OVER_TITLE_SIZE;
    let dims = ui_text(title, size).measure_ui();
    let origin = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let shadow_offset = Vec2::splat(size / 16.0);
    let param = text_param();
    let time = game_over.time();
    for (i, (start, letter)) in title.char_indices().enumerate() {
        let drop = game_over.letter(i);
        if drop <= 0.0 || letter == ' ' {
            continue;
        }
        let x = ui_text(&title[..start], size).measure_ui().x;
        let wave = (time * 4.0 - i as f32 * 0.6).sin() * LETTER_WAVE * drop;
        let y = -(1.0 - ease_out_bounce(drop)) * LETTER_DROP + wave;
        let pos = origin + Vec2::new(x, y);
        let alpha = (drop * 4.0).min(1.0);
        let text = ui_text(&letter.to_string(), size);
        let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * alpha);
        draw(canvas, &text, param.dest(pos + shadow_offset).color(shadow));
        let color = tint(colors[i % colors.len()], TITLE_TINT);
        draw(
            canvas,
            &text,
            param.dest(pos).color(with_alpha(color, alpha)),
        );
    }
    Ok(())
}

pub(super) fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}

pub(super) fn ease_out_bounce(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}
