//! menus by mouse: hover selects, a click chooses, the back button goes back

use ggez::glam::Vec2;
use ggez::input::mouse::{self, MouseButton};
use ggez::Context;

use super::{BlocksState, GameState, Sfx};
use crate::draw::{self, MenuHit, RowPart};
use crate::menus;

impl BlocksState {
    /// cursor hidden during a game, and the button under it timed from when first hovered
    pub(super) fn update_mouse(&mut self, ctx: &mut Context) {
        let hidden = self.state == GameState::Playing && !self.attract.demo();
        if ctx.mouse.cursor_hidden() != hidden {
            mouse::set_cursor_hidden(ctx, hidden);
        }
        let hit = self.cursor.and_then(|at| self.menu_hit(at.x, at.y));
        // a page opened by a click shows the row under the mouse, if any
        if self.mouse_leads {
            if let Some(menu) = self.open_menu() {
                menu.selected = hit.map(|hit| hit.row);
            }
        }
        let last = self.hover.map(|hover| hover.hit);
        if last == hit {
            return;
        }
        // across a row from one arrow to the other
        let arrow_row = |hit: Option<MenuHit>| {
            hit.filter(|hit| matches!(hit.part, RowPart::Step(_)))
                .map(|hit| hit.row)
        };
        if arrow_row(last).is_some() && arrow_row(last) == arrow_row(hit) {
            self.assets.play(Sfx::MenuMove);
        }
        let since = ctx.time.time_since_start().as_secs_f32();
        self.hover = hit.map(|hit| draw::Hover { hit, since });
    }

    /// held: the left button, dragging along a slider sets it
    pub(super) fn mouse_moved(&mut self, x: f32, y: f32, held: bool) {
        self.attract.idle = 0.0;
        self.cursor = Some(Vec2::new(x, y));
        self.mouse_leads = true;
        let hit = self.menu_hit(x, y);
        let row = hit.map(|hit| hit.row);
        let Some(menu) = self.open_menu() else {
            return;
        };
        let moved = row.is_some() && menu.selected != row;
        menu.selected = row;
        if moved {
            self.assets.play(Sfx::MenuMove);
        }
        if let Some(MenuHit {
            part: RowPart::Level(level),
            ..
        }) = hit.filter(|_| held)
        {
            self.set_level(level);
        }
    }

    pub(super) fn mouse_pressed(&mut self, button: MouseButton, x: f32, y: f32) {
        self.mouse_leads = true;
        // a click leaves the demo
        if self.attract_input() {
            return;
        }
        let back = button == MouseButton::Back;
        match self.state {
            // any button, as any key
            GameState::GameOver if self.effects.game_over_ready() => self.end_game_over(),
            GameState::HighScores => self.close_high_scores(),
            GameState::Paused | GameState::Options if back => self.menu_back(),
            _ if button == MouseButton::Left => self.click(x, y),
            _ => {}
        }
    }

    fn click(&mut self, x: f32, y: f32) {
        let Some(MenuHit { row, part }) = self.menu_hit(x, y) else {
            return;
        };
        if let Some(menu) = self.open_menu() {
            menu.selected = Some(row);
        }
        match (self.state, part) {
            (GameState::Menu | GameState::Paused, _) => self.menu_select(),
            (GameState::Options, RowPart::Row) => self.options_enter(),
            (GameState::Options, RowPart::Step(delta)) => self.adjust_option(delta),
            (GameState::Options, RowPart::Level(level)) => self.set_level(level),
            _ => {}
        }
    }

    /// open menu's row under a window point
    fn menu_hit(&self, x: f32, y: f32) -> Option<MenuHit> {
        let point = self.view_settings.view_point(x, y);
        let rect = &self.view_settings.view_rect;
        match self.state {
            GameState::Menu => draw::MenuScreen(&self.menu_state).hit(rect, point),
            GameState::Paused => draw::PausedScreen(&self.paused_state).hit(rect, point),
            GameState::Options => self.options_screen().hit(rect, point),
            _ => None,
        }
    }

    /// selected slider to a level
    fn set_level(&mut self, level: u8) {
        if !self.audio_open {
            return;
        }
        let current = match self.audio_state.selected {
            Some(menus::AUDIO_MUSIC) => self.music_level,
            Some(menus::AUDIO_SOUND) => self.sound_level,
            _ => return,
        };
        if level != current {
            self.adjust_option(level as i8 - current as i8);
        }
    }
}
