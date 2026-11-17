//! paints the screens: background, the board's frames and text, menus and overlays

use std::time::Instant;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::board::{Mode, Score};
use super::debug::{self, DebugView, FrameSample, FrameTiming};
use super::{in_game, OptionsFrom, ScoreView, Screen, Viewer};
use crate::audio::{AudioSettings, MUSIC_THEMES, VOLUME_LEVELS};
use crate::background::Background;
use crate::draw::{self, BoardShown, BoardSprites, Layer, Meshes, OptionRow, ViewSettings};
use crate::effects::BoardFx;
use crate::gfx::{Depth, Vec2};
use crate::menus::Menus;
use crate::render::{Paint, PaintUpload};
use crate::scores::{Initials, ScoreBoard};
use crate::style::Style;
use crate::GameResult;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, show_board.before(BoardSprites))
            .add_systems(PostUpdate, paint.before(PaintUpload));
    }
}

/// everything the screens show
#[derive(SystemParam)]
struct SceneData<'w> {
    screen: Res<'w, State<Screen>>,
    viewer: Res<'w, State<Viewer>>,
    options_from: Res<'w, OptionsFrom>,
    view: Res<'w, ViewSettings>,
    meshes: Res<'w, Meshes>,
    background: Res<'w, Background>,
    style: Res<'w, Style>,
    mode: Res<'w, Mode>,
    score: Res<'w, Score>,
    fx: Res<'w, BoardFx>,
    menus: Res<'w, Menus>,
    audio: Res<'w, AudioSettings>,
    scores: Res<'w, ScoreBoard>,
    score_view: Res<'w, ScoreView>,
    initials: Res<'w, Initials>,
    time: Res<'w, Time<Real>>,
}

impl SceneData<'_> {
    fn shown(&self) -> bool {
        in_game(*self.screen.get(), *self.options_from)
    }

    /// the board, shaking while playing and on game over
    fn game_layer(&self) -> draw::GameLayer<'_> {
        draw::GameLayer {
            style: &self.style,
            assists: self.mode.assists(),
            level_progress: &self.score.level_progress,
            fx: &self.fx,
            level_display: &self.score.level_display,
            score_display: &self.score.score_display,
            best: self.scores.best().max(self.score.score_display.shown()),
            shake: self.shaking().then(|| self.fx.shake()),
        }
    }

    fn shaking(&self) -> bool {
        matches!(self.screen.get(), Screen::Playing | Screen::GameOver)
    }

    /// rows of the options page that is showing
    fn option_rows(&self) -> Vec<OptionRow<'static>> {
        if self.menus.audio_open {
            vec![
                OptionRow::Choice(MUSIC_THEMES[self.audio.theme].0, false),
                OptionRow::Slider(self.audio.music_level),
                OptionRow::Slider(self.audio.sound_level),
                OptionRow::Choice(
                    if self.audio.play_unfocused {
                        "Play"
                    } else {
                        "Pause"
                    },
                    false,
                ),
                OptionRow::Back,
            ]
        } else {
            vec![
                OptionRow::Choice(self.mode.name(), self.options_from.in_game()),
                OptionRow::Choice(self.background.style.name(), false),
                OptionRow::Choice(self.style.block.name(), false),
                OptionRow::Choice(self.style.board.name(), false),
                // opens the audio page
                OptionRow::Link,
                OptionRow::Back,
            ]
        }
    }

    fn draw(&self, list: &mut crate::gfx::DrawList, debug: &DebugView) -> GameResult {
        let view = &*self.view;
        let size = Vec2::new(view.view_rect.w, view.view_rect.h) * view.scale;
        list.begin(size.round(), draw::clear_color(self.background.style));
        list.set_screen_coordinates(view.view_rect);
        let mut painter = draw::Painter::new(list);
        let canvas = &mut painter;
        draw::draw_background(canvas, &self.meshes, &view.view_rect, &self.background);

        let frame = draw::Frame {
            time: self.time.elapsed_secs(),
            meshes: &self.meshes,
            view,
            block: self.style.block,
        };
        if self.shown() {
            self.game_layer().draw(canvas, &frame)?;
        }
        // over the board sprites
        canvas.set_layer(Depth::Front);
        match self.screen.get() {
            Screen::Menu => draw::MenuScreen {
                menu: &self.menus.main,
            }
            .draw(canvas, &frame)?,
            Screen::Playing | Screen::Backdrop => {}
            Screen::Paused => draw::PausedScreen {
                menu: &self.menus.paused,
            }
            .draw(canvas, &frame)?,
            Screen::GameOver => {
                if let Some(game_over) = &self.fx.game_over {
                    draw::GameOverScreen {
                        game_over,
                        score: self.score.score,
                        level: self.score.level,
                    }
                    .draw(canvas, &frame)?;
                }
            }
            Screen::Options => {
                // opened from pause, dim the game underneath
                if self.options_from.in_game() {
                    draw::Overlay.draw(canvas, &frame)?;
                }
                let (title, menu) = if self.menus.audio_open {
                    ("Audio", &self.menus.audio)
                } else {
                    ("Options", &self.menus.options)
                };
                draw::OptionsScreen {
                    title,
                    menu,
                    rows: &self.option_rows(),
                    mode: *self.mode,
                    style: &self.style,
                    max_level: VOLUME_LEVELS,
                }
                .draw(canvas, &frame)?;
            }
            Screen::NameEntry => draw::NameEntryScreen {
                initials: &self.initials,
                score: self.score.score,
            }
            .draw(canvas, &frame)?,
            Screen::HighScores => draw::HighScoresScreen {
                scores: &self.scores,
                highlight: self.score_view.highlight,
            }
            .draw(canvas, &frame)?,
        }

        if *self.viewer.get() == Viewer::On && !debug.hidden {
            let names = debug::Animation::ALL.map(debug::Animation::name);
            draw::DebugScreen {
                names: &names,
                selected: debug.selected,
                speed: debug.speed(),
                paused: debug.paused,
                replay: debug.replay,
                level: debug.level,
                fps: debug.fps_cap(),
                vsync: debug.vsync,
                stats: debug.stats.shown,
                recording: debug.recording.is_some(),
                stats_only: debug.stats_only,
            }
            .draw(canvas, &frame)?;
        }
        Ok(())
    }
}

/// show the board sprites when the board is up, shaking with it
fn show_board(scene: SceneData, mut board: ResMut<BoardShown>) {
    board.shown = scene.shown();
    board.shake = scene
        .shaking()
        .then(|| scene.fx.shake())
        .filter(|_| board.shown);
}

/// paint the frame
fn paint(
    scene: SceneData,
    mut paint: ResMut<Paint>,
    mut debug: ResMut<DebugView>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let start = Instant::now();
    if let Err(e) = scene.draw(&mut paint.0, &debug) {
        log::error!("draw failed: {e}");
    }
    // frame cost for the viewer
    if *scene.viewer.get() == Viewer::On {
        let smoothed = |path| {
            diagnostics
                .get(path)
                .and_then(|d| d.smoothed())
                .unwrap_or_default()
        };
        let timing = FrameTiming {
            fps: smoothed(&FrameTimeDiagnosticsPlugin::FPS),
            frame_ms: smoothed(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
        };
        let sample = FrameSample {
            draws: paint.0.used,
            vertices: paint.0.vertex_count(),
            scene: start.elapsed(),
        };
        let delta = scene.time.delta();
        let fps_cap = debug.fps_cap().unwrap_or(0);
        let scene_info = debug::Scene {
            animation: debug.animation().name(),
            background: scene.background.style.name(),
            block: scene.style.block.name(),
            board: scene.style.board.name(),
            level: debug.level,
            fps_cap,
        };
        debug.record_frame(scene_info, sample, delta, timing);
    }
}
