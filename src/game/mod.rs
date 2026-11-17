//! game states, resources and the fixed rate simulation

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub mod attract;
pub mod bench;
pub mod board;
pub mod debug;
mod input;
mod scene;
pub mod screenshot;
#[cfg(test)]
mod tests;

pub use input::RequestQuit;
pub use screenshot::{save_png, Screenshot};

use crate::audio::{AudioSettings, GameMusic, MusicCue, MusicTempo, MUSIC_THEMES, VOLUME_LEVELS};
use crate::background::{Background, BackgroundStyle};
use crate::controls::GameControls;
use crate::draw::{Meshes, ViewSettings};
use crate::effects::{on_board_events, reset_effects, step_effects};
use crate::menus::Menus;
use crate::scores::{Initials, ScoreBoard};
use crate::settings::Settings;
use crate::style::{self, BlockStyle, BoardStyle, Style};
use board::{Board, BoardEvent, Mode, Play, Score};
use debug::DebugView;

pub const STARTING_LEVEL: usize = 1;
pub const LINES_PER_LEVEL: usize = 10; // lines cleared per level
const MUSIC_LEVEL_SPEEDUP: f32 = 0.02; // tempo gained per level
const MUSIC_MAX_SPEED: f32 = 1.3; // tempo cap
pub const UNFOCUSED_FRAME: std::time::Duration =
    std::time::Duration::from_nanos(1_000_000_000 / 15); // 15fps

/// the screen being shown
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
    Options,
    NameEntry,
    HighScores,
    Backdrop, // the background alone, for the viewer
}

/// the animation viewer, over the screen its animation shows
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Viewer {
    #[default]
    Off,
    On,
}

/// the screen the options return to
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct OptionsFrom(pub Screen);

impl OptionsFrom {
    /// opened from pause, the game shows underneath
    pub fn in_game(&self) -> bool {
        self.0 == Screen::Paused
    }
}

/// whether the window has focus
#[derive(Resource, Debug, Clone, Copy)]
pub struct Focused(pub bool);

/// the high score table's new entry, and whether it follows a game
#[derive(Resource, Debug, Default)]
pub struct ScoreView {
    pub highlight: Option<usize>,
    pub from_game: bool,
}

/// screenshots never write the settings or scores
#[derive(Resource)]
pub struct NoSave;

/// simulation steps, in order
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sim {
    Play,
    React,
    Animate,
}

pub struct GamePlugin {
    pub screenshot: Option<screenshot::Scene>,
    pub settings: Settings,
    pub scores: ScoreBoard,
    pub view: (f32, f32), // drawable size in physical pixels
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let view = ViewSettings::new(self.view.0, self.view.1);
        crate::draw::set_pixel_scale(view.scale);
        app.init_state::<Screen>()
            .init_state::<Viewer>()
            .add_message::<BoardEvent>()
            .insert_resource(Board::new(Mode::Standard))
            .init_resource::<Score>()
            .init_resource::<Mode>()
            .insert_resource(Style::default())
            .insert_resource({
                // the game opens calm at the menu, screenshots are of game scenes
                let mut background = Background::new();
                if self.screenshot.is_none() {
                    background.start_calm();
                }
                background
            })
            .insert_resource(GameControls::default())
            .init_resource::<Menus>()
            .insert_resource(OptionsFrom(Screen::Menu))
            .insert_resource(Focused(true))
            .init_resource::<ScoreView>()
            .insert_resource(Initials::new(&self.settings.initials))
            .insert_resource(self.settings.clone())
            .insert_resource(self.scores.clone())
            .insert_resource(view)
            .insert_resource(Meshes::new())
            .insert_resource(DebugView::new())
            .add_plugins((
                input::InputPlugin,
                scene::ScenePlugin,
                debug::DebugPlugin,
                bench::BenchPlugin,
                attract::AttractPlugin,
            ))
            .add_systems(PreStartup, apply_settings)
            .add_systems(OnEnter(Screen::Menu), new_game)
            .add_systems(
                OnEnter(Screen::GameOver),
                game_over_jingle.run_if(in_state(Viewer::Off)),
            )
            .configure_sets(FixedUpdate, (Sim::Play, Sim::React, Sim::Animate).chain())
            .add_systems(
                FixedUpdate,
                (
                    sim_dt
                        .pipe(play)
                        .in_set(Sim::Play)
                        .run_if(in_state(Screen::Playing))
                        .run_if(in_state(Viewer::Off)),
                    (on_board_events, background_events).in_set(Sim::React),
                    (
                        sim_dt
                            .pipe(|In(dt): In<f64>| dt as f32)
                            .pipe(step_effects)
                            .run_if(effects_run),
                        sim_dt.pipe(update_displays).run_if(displays_run),
                        sim_dt.pipe(step_background),
                        music_tempo,
                        game_music,
                    )
                        .in_set(Sim::Animate),
                ),
            );
        if let Some(scene) = self.screenshot {
            app.insert_resource(NoSave)
                .add_systems(Startup, move |world: &mut World| {
                    screenshot::setup_scene(world, scene)
                });
        }
    }
}

/// seconds this tick simulates, scaled and paused by the viewer
pub fn sim_dt(time: Res<Time>, viewer: Res<State<Viewer>>, debug: Res<DebugView>) -> f64 {
    let dt = time.delta_secs_f64();
    if *viewer.get() == Viewer::Off {
        return dt;
    }
    if debug.paused {
        0.0
    } else {
        dt * debug.speed()
    }
}

fn play(In(dt): In<f64>, mut play: Play) {
    play.step(dt);
}

/// the game is showing, with its board
pub fn in_game(screen: Screen, options_from: OptionsFrom) -> bool {
    match screen {
        Screen::Playing | Screen::Paused | Screen::GameOver | Screen::NameEntry => true,
        Screen::Options => options_from.in_game(),
        _ => false,
    }
}

fn effects_run(screen: Res<State<Screen>>, viewer: Res<State<Viewer>>) -> bool {
    *viewer.get() == Viewer::On
        || matches!(
            screen.get(),
            Screen::Playing | Screen::GameOver | Screen::NameEntry
        )
}

/// score, level and progress bar animations, run on through pause and past the game's end
fn displays_run(
    screen: Res<State<Screen>>,
    viewer: Res<State<Viewer>>,
    options_from: Res<OptionsFrom>,
) -> bool {
    *viewer.get() == Viewer::On || in_game(*screen.get(), *options_from)
}

fn update_displays(In(dt): In<f64>, mut score: ResMut<Score>) {
    score.update_displays(dt);
}

/// the background reacts to clears, level ups and the game's end
fn background_events(mut events: MessageReader<BoardEvent>, mut background: ResMut<Background>) {
    for event in events.read() {
        match *event {
            BoardEvent::LinesCleared { lines } => background.surge(lines),
            BoardEvent::LevelUp => background.level_up(),
            BoardEvent::ToppedOut { level } => background.freeze(level),
            _ => {}
        }
    }
}

/// background at the game's level, calmed behind the title screens
fn step_background(
    In(dt): In<f64>,
    mut background: ResMut<Background>,
    score: Res<Score>,
    screen: Res<State<Screen>>,
    viewer: Res<State<Viewer>>,
    options_from: Res<OptionsFrom>,
) {
    let viewer = *viewer.get() == Viewer::On;
    let level = if viewer || in_game(*screen.get(), *options_from) {
        score.level
    } else {
        1
    };
    // the menu and the screens opened from it
    let title = !viewer
        && match screen.get() {
            Screen::Menu | Screen::HighScores => true,
            Screen::Options => !options_from.in_game(),
            _ => false,
        };
    background.set_calm(title);
    background.update(dt, level);
}

/// faster music in game, normal elsewhere
fn music_tempo(
    score: Res<Score>,
    screen: Res<State<Screen>>,
    viewer: Res<State<Viewer>>,
    options_from: Res<OptionsFrom>,
    mut tempo: ResMut<MusicTempo>,
) {
    let in_game = match screen.get() {
        // the viewer's background alone keeps the level's tempo
        Screen::Playing | Screen::Backdrop => true,
        _ if *viewer.get() == Viewer::On => false,
        Screen::Paused => true,
        Screen::Options => options_from.in_game(),
        _ => false,
    };
    let speed = if in_game {
        music_speed(score.level_display.shown())
    } else {
        1.0
    };
    tempo.set_if_neq(MusicTempo(speed));
}

/// the chosen theme in games through name entry, the demo and the audio options, the title theme elsewhere
fn game_music(
    screen: Res<State<Screen>>,
    options_from: Res<OptionsFrom>,
    menus: Res<Menus>,
    mut music: ResMut<GameMusic>,
) {
    let game = match screen.get() {
        Screen::Playing
        | Screen::Paused
        | Screen::GameOver
        | Screen::NameEntry
        | Screen::Backdrop => true,
        Screen::Options => options_from.in_game() || menus.audio_open,
        _ => false,
    };
    music.set_if_neq(GameMusic(game));
}

/// music playback rate for the shown level
fn music_speed(level: usize) -> f32 {
    (1.0 + (level.max(1) - 1) as f32 * MUSIC_LEVEL_SPEEDUP).min(MUSIC_MAX_SPEED)
}

fn game_over_jingle(mut cues: MessageWriter<MusicCue>) {
    cues.write(MusicCue::GameOver);
}

/// a fresh board, back at the menu
pub fn new_game(
    mut commands: Commands,
    mode: Res<Mode>,
    mut board: ResMut<Board>,
    mut score: ResMut<Score>,
    mut background: ResMut<Background>,
    mut cues: MessageWriter<MusicCue>,
) {
    cues.write(MusicCue::EndGameOver);
    *board = Board::new(*mode);
    *score = Score::default();
    background.reset();
    commands.run_system_cached(reset_effects);
}

/// the saved options, and saving them
#[derive(SystemParam)]
pub struct Prefs<'w> {
    pub settings: ResMut<'w, Settings>,
    pub mode: ResMut<'w, Mode>,
    pub audio: ResMut<'w, AudioSettings>,
    pub background: ResMut<'w, Background>,
    pub style: ResMut<'w, Style>,
    no_save: Option<Res<'w, NoSave>>,
}

impl Prefs<'_> {
    /// apply saved options
    fn apply(&mut self) {
        let settings = &*self.settings;
        if let Some(mode) = style::from_name(&Mode::ALL, &settings.mode, Mode::name) {
            *self.mode = mode;
        }
        let audio = &mut *self.audio;
        if let Some(theme) = MUSIC_THEMES
            .iter()
            .position(|(name, _)| *name == settings.theme)
        {
            audio.theme = theme;
        }
        audio.play_unfocused = settings.play_unfocused;
        audio.music_level = settings.music_level.min(VOLUME_LEVELS);
        audio.sound_level = settings.sound_level.min(VOLUME_LEVELS);
        if let Some(background) = style::from_name(
            &BackgroundStyle::ALL,
            &settings.background,
            BackgroundStyle::name,
        ) {
            self.background.style = background;
        }
        if let Some(block) = style::from_name(&BlockStyle::ALL, &settings.blocks, BlockStyle::name)
        {
            self.style.block = block;
        }
        if let Some(board) = style::from_name(&BoardStyle::ALL, &settings.board, BoardStyle::name) {
            self.style.board = board;
        }
    }

    /// copy the current options into the settings and save them
    pub fn save(&mut self) {
        if self.no_save.is_some() {
            return;
        }
        let audio = &*self.audio;
        let settings = &mut *self.settings;
        settings.mode = self.mode.name().to_string();
        settings.theme = MUSIC_THEMES[audio.theme].0.to_string();
        settings.background = self.background.style.name().to_string();
        settings.blocks = self.style.block.name().to_string();
        settings.board = self.style.board.name().to_string();
        settings.music_level = audio.music_level;
        settings.sound_level = audio.sound_level;
        settings.play_unfocused = audio.play_unfocused;
        settings.save();
    }
}

fn apply_settings(mut prefs: Prefs, mut board: ResMut<Board>) {
    prefs.apply();
    *board = Board::new(*prefs.mode);
}
