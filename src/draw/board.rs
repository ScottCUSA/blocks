use ggez::glam::{IVec2, Vec2};
use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::effects::{Effects, GameOver};
use crate::level_progress::LevelProgress;
use crate::playfield::{self, LineClear, Playfield, SlotState, PLAYFIELD_SIZE, PLAYFIELD_SLOTS};
use crate::rustomino::{Rustomino, RustominoType};
use crate::score_display::ScoreDisplay;
use crate::style::{BlockStyle, BoardStyle};
use crate::util;

use super::*;

pub(super) const BOARD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5); // playfield, staging and side panels
pub(super) const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.06);
pub(super) const DOT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.18);
pub(super) const DOT_SIZE: f32 = 3.0;
pub(super) const CHECKER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.035);
pub(super) const GHOST_FILL_ALPHA: f32 = 0.12;
pub(super) const GHOST_OUTLINE_ALPHA: f32 = 0.6;
pub(super) const SCORE_POP_SCALE: f32 = 0.3; // how much the score grows when it increases
pub(super) const SCORE_POPUP_OFFSET: f32 = 34.0; // where "+points" starts above the score
pub(super) const SCORE_POPUP_RISE: f32 = 40.0; // how far "+points" floats up
pub(super) const QUAD_GLINT_LEAD: f32 = 0.06; // glint peak after a block starts fading, fraction of the clear
pub(super) const QUAD_GLINT_WIDTH: f32 = 0.12; // fraction of the clear
pub(super) const QUAD_GLINT_ROW: f32 = 0.025; // delay per row up
pub(super) const CALLOUT_SIZE: f32 = 32.0;
pub(super) const CALLOUT_QUAD_SIZE: f32 = 48.0;
pub(super) const CALLOUT_POP_SCALE: f32 = 0.5;
pub(super) const TRAIL_SEGMENTS: usize = 8;
pub(super) const TRAIL_ALPHA: f32 = 0.45;
pub(super) const IMPACT_ALPHA: f32 = 0.6;
pub(super) const CURTAIN_FLASH: f32 = 0.6;
pub(super) const PREVIEW_ROW: i32 = playfield::PLAYFIELD_SLOTS[1] as i32; // curtain reaches hold and next last
pub(super) const BEST_OFFSET: f32 = 100.0; // above the score, clear of the popups

/// piece color, grayed and flashed by the game over curtain at row y
pub(super) fn block_color(rtype: RustominoType, game_over: Option<&GameOver>, y: i32) -> Color {
    let color = rtype.color();
    let Some(game_over) = game_over else {
        return color;
    };
    let gray = mix_color(color, util::rgb_to_grayscale(color), game_over.gray(y));
    mix_color(gray, Color::WHITE, CURTAIN_FLASH * game_over.flash(y))
}

/// fade out in a wipe from where the rustomino landed to the edges
pub(super) fn line_clear_alpha(line_clear: &LineClear, x: usize) -> f32 {
    let start = line_clear.wipe_start(x);
    (1.0 - (line_clear.progress() - start) / playfield::LINE_CLEAR_FADE).clamp(0.0, 1.0) as f32
}

/// fixed width outline inside the ghost piece's boundary, as non-overlapping rects
pub(super) fn ghost_outline(view: &ViewSettings, slots: &[IVec2]) -> Vec<Rect> {
    let s = block_size();
    let (p, w) = (CELL - s, snap(GHOST_OUTLINE_WIDTH));
    let has = |x: i32, y: i32| slots.contains(&IVec2::new(x, y));
    let mut rects = Vec::new();
    for &slot in slots {
        let (x, y) = (slot.x, slot.y);
        let r = view.block_rect(slot);
        // slot y+1 is above on screen
        let (l, rt, u, d) = (has(x - 1, y), has(x + 1, y), has(x, y + 1), has(x, y - 1));
        let (ul, ur) = (has(x - 1, y + 1), has(x + 1, y + 1));
        let (dl, dr) = (has(x - 1, y - 1), has(x + 1, y - 1));
        let (right, bottom) = (r.x + s - w, r.y + s - w);
        let top = if u { 0.0 } else { w };
        let side_h = s - top - if d { 0.0 } else { w };
        let edges = [
            // sides, verticals trimmed where they meet horizontals
            (!u, Rect::new(r.x, r.y, s, w)),
            (!d, Rect::new(r.x, bottom, s, w)),
            (!l, Rect::new(r.x, r.y + top, w, side_h)),
            (!rt, Rect::new(right, r.y + top, w, side_h)),
            // inner corners
            (l && u && !ul, Rect::new(r.x, r.y, w, w)),
            (rt && u && !ur, Rect::new(right, r.y, w, w)),
            (l && d && !dl, Rect::new(r.x, bottom, w, w)),
            (rt && d && !dr, Rect::new(right, bottom, w, w)),
            // bridge the padding to the right and above
            (rt && !(u && ur), Rect::new(r.x + s, r.y, p, w)),
            (rt && !(d && dr), Rect::new(r.x + s, bottom, p, w)),
            (u && !(l && ul), Rect::new(r.x, r.y - p, w, p)),
            (u && !(rt && ur), Rect::new(right, r.y - p, w, p)),
        ];
        rects.extend(
            edges
                .into_iter()
                .filter(|(on, _)| *on)
                .map(|(_, rect)| rect),
        );
    }
    rects
}

/// value that counts up and pops when it increases, with rising "+amount"
pub(super) fn draw_counter(canvas: &mut Painter, score: &ScoreDisplay, pos: Vec2) {
    let pop = score.pop();
    let size = UI_FONT_SIZE * (1.0 + SCORE_POP_SCALE * pop);
    let grow = Vec2::new(0.0, (size - UI_FONT_SIZE) / 2.0);
    let color = mix_color(Color::WHITE, ACCENT_COLOR, pop);
    draw_text(canvas, &score.shown().to_string(), pos - grow, size, color);

    for popup in &score.popups {
        let t = popup.progress();
        let rise = Vec2::new(0.0, SCORE_POPUP_OFFSET + SCORE_POPUP_RISE * t);
        let color = with_alpha(ACCENT_COLOR, 1.0 - t * t);
        let text = format!("+{}", popup.points);
        draw_text(canvas, &text, pos - rise, UI_FONT_SIZE * 0.8, color);
    }
}

/// the board with its previews, progress, effects and text
pub struct GameLayer<'a> {
    pub playfield: &'a Playfield,
    pub next: &'a Option<Rustomino>,
    pub held: &'a Option<Rustomino>,
    pub game_over: Option<&'a GameOver>,
    pub board: BoardStyle,
    pub assists: bool, // ghost and hold
    pub level_progress: &'a LevelProgress,
    pub effects: &'a Effects,
    pub level_display: &'a ScoreDisplay,
    pub score_display: &'a ScoreDisplay,
    pub best: usize,
    pub shake: Option<Vec2>, // None when not shaking
}

impl Layer for GameLayer<'_> {
    fn draw(&self, canvas: &mut Painter) -> GameResult {
        let screen = canvas.screen_coordinates();
        if let (Some(rect), Some(shake)) = (screen, self.shake) {
            let shaken = Rect::new(rect.x - shake.x, rect.y - shake.y, rect.w, rect.h);
            set_screen_coordinates(canvas, shaken);
        }
        self.draw_backgrounds(canvas);
        self.draw_playfield(canvas);
        if self.assists {
            self.draw_preview(canvas, self.held, canvas.view.hold_rect);
        }
        self.draw_preview(canvas, self.next, canvas.view.preview_rect);
        self.draw_level_progress(canvas);
        self.draw_effects(canvas)?;
        self.draw_labels(canvas);
        if let Some(rect) = screen {
            set_screen_coordinates(canvas, rect);
        }
        Ok(())
    }
}

impl GameLayer<'_> {
    /// frames, panel backgrounds and the board pattern
    fn draw_backgrounds(&self, canvas: &mut Painter) {
        let view = canvas.view;
        let (staging, playfield) = (view.staging_rect, view.playfield_rect);
        let play_area = Rect::new(
            staging.x,
            staging.y,
            staging.w,
            playfield.bottom() - staging.y,
        );
        let panels = [
            Some(view.preview_rect),
            self.assists.then_some(view.hold_rect),
        ];
        for rect in [Some(play_area)].into_iter().chain(panels).flatten() {
            draw_rect(canvas, frame_rect(rect), FRAME_COLOR);
        }
        for rect in [Some(staging), Some(playfield)]
            .into_iter()
            .chain(panels)
            .flatten()
        {
            draw_rect(canvas, rect, BOARD_BACKGROUND_COLOR);
        }

        let (cols, rows) = (PLAYFIELD_SLOTS[0], PLAYFIELD_SIZE[1] as usize);
        // the gap above a row
        let gap_y = |row: usize| playfield.bottom() - 2.0 * BLOCK_PADDING - row as f32 * CELL;
        match self.board {
            BoardStyle::Grid => {
                // lines in the gaps between cells
                for col in 1..cols {
                    let (x, w) = snap_line(playfield.x + col as f32 * CELL);
                    draw_rect(
                        canvas,
                        Rect::new(x, playfield.y, w, playfield.h),
                        GRID_COLOR,
                    );
                }
                for row in 1..rows {
                    let (y, h) = snap_line(gap_y(row));
                    draw_rect(
                        canvas,
                        Rect::new(playfield.x, y, playfield.w, h),
                        GRID_COLOR,
                    );
                }
            }
            BoardStyle::Dots => {
                // dots where the gaps cross
                for row in 1..rows {
                    for col in 1..cols {
                        let at = Vec2::new(playfield.x + col as f32 * CELL, gap_y(row))
                            + (BLOCK_PADDING - DOT_SIZE) / 2.0;
                        let dot = snap_rect(Rect::new(at.x, at.y, DOT_SIZE, DOT_SIZE));
                        draw_rect(canvas, dot, DOT_COLOR);
                    }
                }
            }
            BoardStyle::Checker => {
                for row in 0..rows as i32 {
                    for col in (row % 2..cols as i32).step_by(2) {
                        draw_rect(canvas, view.block_rect(IVec2::new(col, row)), CHECKER_COLOR);
                    }
                }
            }
            BoardStyle::Clean => {}
        }
    }

    /// locked blocks, then the active piece and its ghost
    fn draw_playfield(&self, canvas: &mut Painter) {
        let (view, playfield, game_over) = (canvas.view, self.playfield, self.game_over);
        for (y, row) in playfield.slots.iter().enumerate() {
            let clearing = playfield
                .line_clear
                .as_ref()
                .filter(|lc| lc.lines.contains(&y));
            for (x, slot) in row.iter().enumerate() {
                let SlotState::Locked(rtype) = slot else {
                    continue;
                };
                let mut color = block_color(*rtype, game_over, y as i32);
                if let Some(line_clear) = clearing {
                    let progress = line_clear.progress() as f32;
                    let flash = (1.0 - progress * 4.0).max(0.0);
                    let base = if line_clear.lines.len() >= 4 {
                        // glint riding the wipe outward
                        let bottom = line_clear.lines.iter().min().copied().unwrap_or(y);
                        let peak = line_clear.wipe_start(x) as f32
                            + QUAD_GLINT_LEAD
                            + (y - bottom) as f32 * QUAD_GLINT_ROW;
                        let glint = (1.0 - (progress - peak).abs() / QUAD_GLINT_WIDTH).max(0.0);
                        mix_color(LINE_CLEAR_COLOR, Color::WHITE, glint * 0.8)
                    } else {
                        LINE_CLEAR_COLOR
                    };
                    color = mix_color(base, Color::WHITE, flash);
                    color.a *= line_clear_alpha(line_clear, x);
                }
                draw_block(
                    canvas,
                    view.block_rect(IVec2::new(x as i32, y as i32)),
                    color,
                );
            }
        }

        if let Some(active) = &playfield.active_rustomino {
            for block in active.playfield_slots() {
                let color = block_color(active.rtype, game_over, block.y);
                draw_block(canvas, view.block_rect(block), color);
            }
        }

        let Some(ghost) = playfield.ghost_rustomino.as_ref().filter(|_| self.assists) else {
            return;
        };
        let color = block_color(ghost.rtype, game_over, PREVIEW_ROW);
        let fill = with_alpha(color, GHOST_FILL_ALPHA);
        let outline = with_alpha(color, GHOST_OUTLINE_ALPHA);
        let slots = ghost.playfield_slots();
        if canvas.block == BlockStyle::Jelly {
            // rounded cells like the blocks
            for block in slots {
                draw_rounded(canvas, view.block_rect(block), fill);
                draw_rounded_outline(canvas, view.block_rect(block), outline);
            }
            return;
        }
        for block in slots {
            draw_rect(canvas, view.block_rect(block), fill);
        }
        for rect in ghost_outline(view, &slots) {
            draw_rect(canvas, rect, outline);
        }
    }

    /// a piece centered in a side panel
    fn draw_preview(&self, canvas: &mut Painter, rustomino: &Option<Rustomino>, panel: Rect) {
        let Some(rustomino) = rustomino else {
            return;
        };
        let blocks = rustomino.blocks;
        let min = blocks.iter().fold(blocks[0], |a, b| a.min(*b));
        let max = blocks.iter().fold(blocks[0], |a, b| a.max(*b));
        let width = (max.x - min.x + 1) as f32 * CELL - BLOCK_PADDING;
        let height = (max.y - min.y + 1) as f32 * CELL - BLOCK_PADDING;
        let left = snap(panel.x + (panel.w - width) / 2.0);
        let top = snap(panel.y + (panel.h - height) / 2.0);
        let color = block_color(rustomino.rtype, self.game_over, PREVIEW_ROW);
        for block in blocks {
            let x = left + (block.x - min.x) as f32 * CELL;
            let y = top + (max.y - block.y) as f32 * CELL;
            draw_block(canvas, Rect::new(x, y, block_size(), block_size()), color);
        }
    }

    fn draw_labels(&self, canvas: &mut Painter) {
        let view = canvas.view;
        let small = UI_FONT_SIZE * 0.75;
        let above = |panel: Rect| Vec2::new(panel.x, panel.y - (small + FRAME_WIDTH + 8.0));
        draw_piece_colored(canvas, "Blocks!", view.title_pos, UI_FONT_SIZE);
        if self.assists {
            draw_text(canvas, "Hold", above(view.hold_rect), small, LABEL_COLOR);
        }
        draw_text(canvas, "Next", above(view.preview_rect), small, LABEL_COLOR);
        draw_text(
            canvas,
            "Level:",
            view.level_label_pos,
            UI_FONT_SIZE,
            LABEL_COLOR,
        );
        draw_text(
            canvas,
            "Score:",
            view.score_label_pos,
            UI_FONT_SIZE,
            LABEL_COLOR,
        );
        draw_counter(canvas, self.level_display, view.level_pos);
        draw_counter(canvas, self.score_display, view.score_pos);
        let best = Vec2::new(0.0, -BEST_OFFSET);
        draw_text(
            canvas,
            "Best:",
            view.score_label_pos + best,
            small,
            LABEL_COLOR,
        );
        let score = self.best.to_string();
        draw_text(canvas, &score, view.score_pos + best, small, Color::WHITE);
    }

    /// segments under the playfield filled per line toward the next level
    fn draw_level_progress(&self, canvas: &mut Painter) {
        let progress = self.level_progress;
        let bar = canvas.view.progress_rect;
        let frame_color = mix_color(FRAME_COLOR, ACCENT_COLOR, progress.burst());
        draw_rect(canvas, frame_rect(bar), frame_color);
        draw_rect(canvas, bar, BOARD_BACKGROUND_COLOR);

        // whole pixel segments and gaps, centered in the bar
        let per_level = progress.len();
        let gap = snap(BLOCK_PADDING).max(1.0 / pixel_scale());
        let pitch = snap((bar.w - gap) / per_level as f32);
        let left = bar.x + snap((bar.w - gap - pitch * per_level as f32) / 2.0) + gap;
        let (top, height) = (snap(bar.y + BLOCK_PADDING), snap(PROGRESS_HEIGHT));
        let center_y = top + height / 2.0;
        let pulse = progress.pulse();
        for i in 0..per_level {
            let rect = Rect::new(left + i as f32 * pitch, top, pitch - gap, height);
            draw_rect(canvas, rect, SLIDER_EMPTY_COLOR);
            if let Some((_, amount)) = pulse.filter(|(at, _)| *at == i) {
                draw_rect(canvas, rect, with_alpha(ACCENT_COLOR, amount * 0.35));
            }
            let Some(look) = progress.segment(i) else {
                continue;
            };
            let (w, h) = (rect.w * look.width, rect.h * look.height);
            let sized = snap_rect(Rect::new(
                rect.center().x - w / 2.0,
                center_y - h / 2.0,
                w,
                h,
            ));
            // soft two layer halo
            for (spread, alpha) in [(4.0, 0.12), (2.0, 0.22)] {
                if look.glow <= 0.0 {
                    break;
                }
                let halo = snap_rect(inset(sized, -spread * (0.5 + look.glow)));
                draw_rect(canvas, halo, with_alpha(ACCENT_COLOR, look.glow * alpha));
            }
            let color = mix_color(ACCENT_COLOR, Color::WHITE, look.heat);
            draw_block(canvas, sized, color);
        }

        for spark in &progress.sparks {
            let size = snap(spark.size);
            let x = left + spark.pos.x * pitch - gap / 2.0 - size / 2.0;
            let y = center_y + spark.pos.y - size / 2.0;
            let color = if spark.white {
                Color::WHITE
            } else {
                ACCENT_COLOR
            };
            let rect = snap_rect(Rect::new(x, y, size, size));
            draw_rect(canvas, rect, with_alpha(color, spark.fade()));
        }
    }

    /// particles, playfield flash and line clear callout
    fn draw_effects(&self, canvas: &mut Painter) -> GameResult {
        let (view, effects) = (canvas.view, self.effects);
        let playfield = view.playfield_rect;
        let flash = effects.flash();
        if flash > 0.0 {
            draw_rect(canvas, playfield, with_alpha(Color::WHITE, flash));
        }
        let left = view.staging_rect.x + 1.0;
        let bottom = playfield.bottom() - 1.0;
        for trail in &effects.trails {
            let fade = trail.fade();
            let x = left + trail.x * CELL;
            let length = trail.from - trail.to;
            for i in 0..TRAIL_SEGMENTS {
                let t = i as f32 / TRAIL_SEGMENTS as f32;
                let y0 = trail.to + length * t;
                let h = length / TRAIL_SEGMENTS as f32 * CELL;
                let rect = snap_edges(Rect::new(x, bottom - y0 * CELL - h, CELL, h));
                let alpha = TRAIL_ALPHA * trail.strength * fade * (1.0 - t);
                draw_rect(canvas, rect, with_alpha(trail.color, alpha));
            }
        }
        if let Some(impact) = &effects.impact {
            let color = with_alpha(Color::WHITE, IMPACT_ALPHA * impact.strength * impact.fade());
            for block in impact.blocks {
                draw_rect(canvas, view.block_rect(block), color);
            }
        }
        for particle in effects.particles.iter().filter(|p| p.visible()) {
            let fade = particle.fade();
            let size = BLOCK_SIZE * particle.size * (0.4 + 0.6 * fade);
            let center = Vec2::new(left + particle.pos.x * CELL, bottom - particle.pos.y * CELL);
            let rect = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
            draw_rect(canvas, snap_rect(rect), with_alpha(particle.color, fade));
        }

        let Some(callout) = &effects.callout else {
            return Ok(());
        };
        let t = callout.progress();
        let quad = callout.lines >= 4;
        let base = if quad {
            CALLOUT_QUAD_SIZE
        } else {
            CALLOUT_SIZE
        };
        let mut size = base * (1.0 + CALLOUT_POP_SCALE * callout.pop());
        let width = ui_text(callout.text(), size).measure_ui(canvas)?.x;
        size *= ((playfield.w - 2.0 * PANEL_PADDING) / width).min(1.0);
        let alpha = ((1.0 - t) / 0.4).min(1.0);
        let center = Vec2::new(playfield.center().x, bottom - (callout.row + t) * CELL);
        if quad {
            let (text, shadow) = piece_colored_text(callout.text(), size, alpha);
            let pos = center - text.measure_ui(canvas)? / 2.0;
            draw(canvas, &shadow, text_param().dest(pos + Vec2::splat(3.0)));
            draw(canvas, &text, text_param().dest(pos));
        } else {
            let color = with_alpha(LINE_CLEAR_COLOR, alpha);
            draw_text_centered(canvas, &ui_text(callout.text(), size), center, color)?;
        }
        Ok(())
    }
}
