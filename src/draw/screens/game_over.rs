use ggez::glam::Vec2;
use ggez::graphics::Color;
use ggez::GameResult;

use crate::effects::GameOver;
use crate::rustomino::RustominoType;
use crate::util;
use strum::IntoEnumIterator;

use super::*;

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
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let Self {
            game_over,
            score,
            level,
        } = *self;
        let view_rect = canvas.view.view_rect;
        draw_overlay(canvas, game_over.overlay());
        let drop = game_over.panel();
        if drop <= 0.0 {
            return Ok(());
        }
        let time = game_over.time();
        let height = GAME_OVER_TITLE_ROW + 2.0 * GAME_OVER_STAT_ROW + PANEL_PADDING;
        let mut panel = centered_panel(&view_rect, height);
        panel.y -= (1.0 - util::ease_out_back(drop)) * view_rect.h;
        let top = panel.y;
        draw_panel(canvas, panel);

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
            let label_y = center_y - label.measure_ui(canvas)?.y / 2.0;
            draw_shadowed(canvas, &label, Vec2::new(left, label_y), LABEL_COLOR);
            let value = ui_text(&value.to_string(), GAME_OVER_STAT_SIZE);
            let dims = value.measure_ui(canvas)?;
            let counting = game_over.count() > 0.0 && game_over.count() < 1.0;
            let color = if counting { ACCENT_COLOR } else { Color::WHITE };
            let pos = Vec2::new(right - dims.x, center_y - dims.y / 2.0);
            draw_shadowed(canvas, &value, pos, color);
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
    let origin = center - ui_text(title, size).measure_ui(canvas)? / 2.0;
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let shadow_offset = Vec2::splat(size / 16.0);
    let param = text_param();
    let time = game_over.time();
    for (i, (start, letter)) in title.char_indices().enumerate() {
        let drop = game_over.letter(i);
        if drop <= 0.0 || letter == ' ' {
            continue;
        }
        let x = ui_text(&title[..start], size).measure_ui(canvas)?.x;
        let wave = (time * 4.0 - i as f32 * 0.6).sin() * LETTER_WAVE * drop;
        let y = -(1.0 - ease_out_bounce(drop)) * LETTER_DROP + wave;
        let pos = origin + Vec2::new(x, y);
        let alpha = (drop * 4.0).min(1.0);
        let text = ui_text(&letter.to_string(), size);
        let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * alpha);
        draw(canvas, &text, param.dest(pos + shadow_offset).color(shadow));
        let color = with_alpha(tint(colors[i % colors.len()], TITLE_TINT), alpha);
        draw(canvas, &text, param.dest(pos).color(premultiply(color)));
    }
    Ok(())
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
