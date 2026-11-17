//! the game's systems driven headless, with key presses and a manual clock

use std::time::Duration;

use bevy::diagnostic::DiagnosticsPlugin;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::WindowFocused;

use super::board::{Board, Score};
use super::{GamePlugin, Screen};
use crate::audio::AudioPlugin;
use crate::draw::BoardShown;
use crate::effects::{BoardFx, EffectsPlugin, Particle};
use crate::playfield::SlotState;
use crate::render::Paint;
use crate::scores::ScoreBoard;
use crate::settings::Settings;
use crate::Resources;

const FRAME: Duration = Duration::from_millis(16);

fn app() -> App {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
    let resources = Resources { dirs: vec![dir] };
    if let Ok(font) = resources.load("04b30.ttf") {
        // already set by another test
        let _ = crate::gfx::set_font(font);
    }
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        bevy::input::InputPlugin,
        DiagnosticsPlugin,
    ))
    .add_message::<WindowFocused>()
    .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME))
    .insert_resource(Time::<Fixed>::from_hz(240.0))
    .insert_resource(resources)
    .init_resource::<Paint>()
    .init_resource::<BoardShown>()
    .add_plugins((
        AudioPlugin { silent: true },
        EffectsPlugin,
        GamePlugin {
            screenshot: None,
            settings: Settings::default(),
            scores: ScoreBoard::default(),
            view: (1024.0, 768.0),
        },
    ))
    .insert_resource(super::NoSave);
    app.update();
    app
}

fn press(app: &mut App, key: KeyCode) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }
    app.update();
}

fn frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

fn screen(app: &App) -> Screen {
    *app.world().resource::<State<Screen>>().get()
}

fn locked(app: &App) -> usize {
    let board = app.world().resource::<Board>();
    board
        .playfield
        .slots
        .iter()
        .flatten()
        .filter(|slot| matches!(slot, SlotState::Locked(_)))
        .count()
}

/// start a game from the menu, with a piece falling
fn start(app: &mut App) {
    assert_eq!(screen(app), Screen::Menu);
    press(app, KeyCode::Enter);
    frames(app, 2);
    assert_eq!(screen(app), Screen::Playing);
    assert!(app
        .world()
        .resource::<Board>()
        .playfield
        .active_rustomino
        .is_some());
}

#[test]
fn hard_drop_locks_and_scores() {
    let mut app = app();
    start(&mut app);
    press(&mut app, KeyCode::Space);
    frames(&mut app, 2);
    assert_eq!(locked(&app), 4);
    assert!(app.world().resource::<Score>().score > 0);
    // dust from the landing, as entities
    let mut particles = app.world_mut().query::<&Particle>();
    assert!(particles.iter(app.world()).count() > 0);
    // the next piece comes in
    frames(&mut app, 2);
    assert!(app
        .world()
        .resource::<Board>()
        .playfield
        .active_rustomino
        .is_some());
}

#[test]
fn gravity_moves_the_piece_down() {
    let mut app = app();
    start(&mut app);
    let row = |app: &App| {
        app.world()
            .resource::<Board>()
            .playfield
            .active_lowest_row()
            .unwrap()
    };
    let before = row(&app);
    // level 1 falls a row every 0.8s
    frames(&mut app, 120);
    assert!(row(&app) < before);
}

#[test]
fn pause_and_resume() {
    let mut app = app();
    start(&mut app);
    press(&mut app, KeyCode::Escape);
    frames(&mut app, 1);
    assert_eq!(screen(&app), Screen::Paused);
    let row = app
        .world()
        .resource::<Board>()
        .playfield
        .active_lowest_row();
    frames(&mut app, 120);
    // nothing falls while paused
    assert_eq!(
        app.world()
            .resource::<Board>()
            .playfield
            .active_lowest_row(),
        row
    );
    press(&mut app, KeyCode::Escape);
    frames(&mut app, 1);
    assert_eq!(screen(&app), Screen::Playing);
}

#[test]
fn topping_out_ends_the_game() {
    let mut app = app();
    start(&mut app);
    // hard drops stack up until a piece can't spawn
    for _ in 0..40 {
        if screen(&app) != Screen::Playing {
            break;
        }
        press(&mut app, KeyCode::Space);
        frames(&mut app, 30);
    }
    assert_eq!(screen(&app), Screen::GameOver);
    assert!(app.world().resource::<BoardFx>().game_over.is_some());
    // keys wait for the overlay, then the score goes on the board
    frames(&mut app, 400);
    press(&mut app, KeyCode::Enter);
    frames(&mut app, 2);
    assert_eq!(screen(&app), Screen::NameEntry);
    press(&mut app, KeyCode::Enter);
    frames(&mut app, 2);
    assert_eq!(screen(&app), Screen::HighScores);
    assert_eq!(app.world().resource::<ScoreBoard>().scores.len(), 1);
    // any key starts over at the menu with a fresh board
    press(&mut app, KeyCode::Enter);
    frames(&mut app, 2);
    assert_eq!(screen(&app), Screen::Menu);
    assert_eq!(locked(&app), 0);
    assert_eq!(app.world().resource::<Score>().score, 0);
    let mut particles = app.world_mut().query::<&Particle>();
    assert_eq!(particles.iter(app.world()).count(), 0);
}

#[test]
fn options_from_pause_return_to_pause() {
    let mut app = app();
    start(&mut app);
    press(&mut app, KeyCode::Escape);
    frames(&mut app, 1);
    // Options is the second pause item
    press(&mut app, KeyCode::ArrowDown);
    press(&mut app, KeyCode::Enter);
    frames(&mut app, 1);
    assert_eq!(screen(&app), Screen::Options);
    press(&mut app, KeyCode::Escape);
    frames(&mut app, 1);
    assert_eq!(screen(&app), Screen::Paused);
    // the game is still there
    assert!(app
        .world()
        .resource::<Board>()
        .playfield
        .active_rustomino
        .is_some());
}

#[test]
fn viewer_plays_every_animation() {
    use super::debug::Animation;
    use super::Viewer;
    let mut app = app();
    press(&mut app, KeyCode::F12);
    frames(&mut app, 2);
    assert_eq!(*app.world().resource::<State<Viewer>>().get(), Viewer::On);
    for _ in 0..Animation::ALL.len() {
        frames(&mut app, 200);
        press(&mut app, KeyCode::ArrowDown);
    }
    press(&mut app, KeyCode::Escape);
    frames(&mut app, 2);
    assert_eq!(*app.world().resource::<State<Viewer>>().get(), Viewer::Off);
    assert_eq!(screen(&app), Screen::Menu);
    assert_eq!(locked(&app), 0);
}
