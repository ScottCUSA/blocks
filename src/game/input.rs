//! keyboard handling, one system per screen

use bevy::app::{RunFixedMainLoop, RunFixedMainLoopSystems};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::WindowFocused;

use super::board::{Mode, Play, Score};
use super::debug::OpenViewer;
use super::{Focused, NoSave, OptionsFrom, Prefs, ScoreView, Screen, Viewer};
use crate::audio::{MusicCue, PlaySfx, Sfx, MUSIC_THEMES, VOLUME_LEVELS};
use crate::background::Background;
use crate::controls;
use crate::menus::{self, Menu, Menus};
use crate::scores::{HighScore, Initials, ScoreBoard};
use crate::style::{self, BlockStyle, BoardStyle};

/// a key press this frame, `text` is what it types
#[derive(Debug, Clone, Copy)]
pub struct KeyPress {
    pub key: KeyCode,
    pub repeat: bool,
    pub text: Option<char>,
}

/// this frame's key presses and releases, in order
#[derive(Resource, Default)]
pub struct Keys {
    pub pressed: Vec<KeyPress>,
    pub released: Vec<KeyCode>,
    pub shift: bool,
}

impl Keys {
    /// first presses, without repeats
    fn first(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.pressed.iter().filter(|p| !p.repeat).map(|p| p.key)
    }
}

/// save and exit
#[derive(Message, Debug, Clone, Copy)]
pub struct RequestQuit;

/// handles keys after they are collected, before the fixed ticks
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HandleKeys;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Keys>()
            .add_message::<RequestQuit>()
            .configure_sets(
                RunFixedMainLoop,
                HandleKeys.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(
                RunFixedMainLoop,
                (
                    (collect_keys, focus).chain(),
                    (
                        playing.run_if(in_state(Screen::Playing)),
                        menu.run_if(in_state(Screen::Menu)),
                        paused.run_if(in_state(Screen::Paused)),
                        game_over.run_if(in_state(Screen::GameOver)),
                        options.run_if(in_state(Screen::Options)),
                        name_entry.run_if(in_state(Screen::NameEntry)),
                        high_scores.run_if(in_state(Screen::HighScores)),
                    )
                        .run_if(in_state(Viewer::Off))
                        .run_if(super::attract::player),
                )
                    .chain()
                    .in_set(HandleKeys),
            );
    }
}

pub(super) fn collect_keys(
    mut keys: ResMut<Keys>,
    mut events: MessageReader<KeyboardInput>,
    input: Res<ButtonInput<KeyCode>>,
) {
    keys.pressed.clear();
    keys.released.clear();
    keys.shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    for event in events.read() {
        let Some(key) = controls::keycode(event.key_code) else {
            continue;
        };
        match event.state {
            ButtonState::Pressed => keys.pressed.push(KeyPress {
                key,
                repeat: event.repeat,
                text: event.text.as_ref().and_then(|t| t.chars().next()),
            }),
            ButtonState::Released => keys.released.push(key),
        }
    }
}

/// pause a game when the window loses focus
pub(super) fn focus(
    mut events: MessageReader<WindowFocused>,
    mut focused: ResMut<Focused>,
    screen: Res<State<Screen>>,
    viewer: Res<State<Viewer>>,
    no_save: Option<Res<NoSave>>,
    attract: Res<super::attract::Attract>,
    mut play: Play,
) {
    for event in events.read() {
        if no_save.is_some() || event.focused == focused.0 {
            continue;
        }
        focused.0 = event.focused;
        // a demo plays on
        if !event.focused
            && *screen.get() == Screen::Playing
            && *viewer.get() == Viewer::Off
            && !attract.demo()
        {
            play.pause();
        }
    }
}

fn is_enter(key: KeyCode) -> bool {
    matches!(key, KeyCode::Enter | KeyCode::NumpadEnter)
}

fn playing(keys: Res<Keys>, mut play: Play) {
    for press in &keys.pressed {
        if play.board.topped_out {
            return;
        }
        // pause at once, dropping held controls
        if press.key == KeyCode::Escape {
            play.pause();
        }
        if !press.repeat {
            if let Some(&control) = play.controls.key_map.get(&press.key) {
                play.control(control);
            }
            play.controls.set_pressed(Some(press.key));
        }
    }
    for &key in &keys.released {
        play.controls.set_released(Some(key));
    }
}

fn menu(
    keys: Res<Keys>,
    mut menus: ResMut<Menus>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
    mut options_from: ResMut<OptionsFrom>,
    mut score_view: ResMut<ScoreView>,
    mut quit: MessageWriter<RequestQuit>,
    mut viewer: MessageWriter<OpenViewer>,
) {
    for key in keys.first() {
        match key {
            key if is_enter(key) => {
                sfx.write(PlaySfx(Sfx::MenuSelect));
                match menus.main.selected() {
                    0 => {
                        next.set(Screen::Playing);
                        menus.main.reset_selection();
                    }
                    1 => {
                        options_from.0 = Screen::Menu;
                        next.set(Screen::Options);
                    }
                    2 => {
                        score_view.from_game = false;
                        next.set(Screen::HighScores);
                    }
                    _ => {
                        quit.write(RequestQuit);
                    }
                }
            }
            KeyCode::Escape => {
                quit.write(RequestQuit);
            }
            KeyCode::F12 => {
                viewer.write(OpenViewer);
            }
            KeyCode::ArrowUp => {
                menus.main.previous();
                sfx.write(PlaySfx(Sfx::MenuMove));
            }
            KeyCode::ArrowDown => {
                menus.main.next();
                sfx.write(PlaySfx(Sfx::MenuMove));
            }
            _ => {}
        }
    }
}

fn paused(
    keys: Res<Keys>,
    mut menus: ResMut<Menus>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
    mut options_from: ResMut<OptionsFrom>,
    mut quit: MessageWriter<RequestQuit>,
) {
    for key in keys.first() {
        match key {
            KeyCode::Escape => {
                menus.paused.reset_selection();
                next.set(Screen::Playing);
            }
            key if is_enter(key) => {
                sfx.write(PlaySfx(Sfx::MenuSelect));
                match menus.paused.selected() {
                    0 => {
                        next.set(Screen::Playing);
                        menus.paused.reset_selection();
                    }
                    1 => {
                        options_from.0 = Screen::Paused;
                        next.set(Screen::Options);
                    }
                    2 => {
                        // a new game starts at the menu
                        next.set(Screen::Menu);
                        menus.paused.reset_selection();
                    }
                    _ => {
                        quit.write(RequestQuit);
                    }
                }
            }
            KeyCode::ArrowUp => {
                menus.paused.previous();
                sfx.write(PlaySfx(Sfx::MenuMove));
            }
            KeyCode::ArrowDown => {
                menus.paused.next();
                sfx.write(PlaySfx(Sfx::MenuMove));
            }
            _ => {}
        }
    }
}

/// any key, once the overlay is in, leaves for name entry or the score board
fn game_over(
    keys: Res<Keys>,
    fx: Res<crate::effects::BoardFx>,
    score: Res<Score>,
    scores: Res<ScoreBoard>,
    settings: Res<crate::settings::Settings>,
    mut initials: ResMut<Initials>,
    mut background: ResMut<Background>,
    mut score_view: ResMut<ScoreView>,
    mut sfx: MessageWriter<PlaySfx>,
    mut cues: MessageWriter<MusicCue>,
    mut next: ResMut<NextState<Screen>>,
) {
    if keys.first().next().is_none() || !fx.game_over_ready() {
        return;
    }
    cues.write(MusicCue::EndGameOver);
    background.thaw();
    score_view.highlight = None;
    score_view.from_game = true;
    if scores.qualifies(score.score) {
        *initials = Initials::new(&settings.initials);
        sfx.write(PlaySfx(Sfx::HighScore));
        next.set(Screen::NameEntry);
    } else {
        sfx.write(PlaySfx(Sfx::MenuSelect));
        next.set(Screen::HighScores);
    }
}

fn options(
    keys: Res<Keys>,
    mut menus: ResMut<Menus>,
    mut prefs: Prefs,
    options_from: Res<OptionsFrom>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
) {
    for press in &keys.pressed {
        let (key, repeat) = (press.key, press.repeat);
        let audio_open = menus.audio_open;
        let (menu, back) = if audio_open {
            (&mut menus.audio, menus::AUDIO_BACK)
        } else {
            (&mut menus.options, menus::OPTION_BACK)
        };
        let on_back = menu.selected() == back;
        let on_audio = !audio_open && menu.selected() == menus::OPTION_AUDIO;
        let enter = is_enter(key);
        if (key == KeyCode::Escape || (enter && on_back)) && !repeat {
            sfx.write(PlaySfx(Sfx::MenuSelect));
            if audio_open {
                // back to the options, on the audio row
                menus.audio_open = false;
            } else {
                prefs.save();
                menus.options.reset_selection();
                menus.audio_open = false;
                next.set(options_from.0);
            }
        } else if enter && on_audio && !repeat {
            open_audio(&mut menus, &mut sfx);
        } else if enter && audio_open && !repeat {
            // enter flips on/off settings too
            if menus.audio.selected() == menus::AUDIO_UNFOCUSED {
                adjust_option(1, &mut menus, &mut prefs, &options_from, &mut sfx);
            }
        } else if key == KeyCode::ArrowUp && !repeat {
            menu.previous();
            sfx.write(PlaySfx(Sfx::MenuMove));
        } else if key == KeyCode::ArrowDown && !repeat {
            menu.next();
            sfx.write(PlaySfx(Sfx::MenuMove));
        } else if key == KeyCode::ArrowLeft {
            adjust_option(-1, &mut menus, &mut prefs, &options_from, &mut sfx);
        } else if key == KeyCode::ArrowRight {
            adjust_option(1, &mut menus, &mut prefs, &options_from, &mut sfx);
        }
    }
}

fn open_audio(menus: &mut Menus, sfx: &mut MessageWriter<PlaySfx>) {
    menus.audio_open = true;
    menus.audio.reset_selection();
    sfx.write(PlaySfx(Sfx::MenuSelect));
}

/// change the selected option's setting
fn adjust_option(
    delta: i8,
    menus: &mut Menus,
    prefs: &mut Prefs,
    options_from: &OptionsFrom,
    sfx: &mut MessageWriter<PlaySfx>,
) {
    let step = |level: u8| (level as i8 + delta).clamp(0, VOLUME_LEVELS as i8) as u8;
    if menus.audio_open {
        let audio = &mut *prefs.audio;
        match menus.audio.selected() {
            menus::AUDIO_THEME => {
                // the new theme starts playing, no click over it
                let count = MUSIC_THEMES.len() as isize;
                audio.theme = (audio.theme as isize + delta as isize).rem_euclid(count) as usize;
                return;
            }
            menus::AUDIO_MUSIC => audio.music_level = step(audio.music_level),
            menus::AUDIO_SOUND => audio.sound_level = step(audio.sound_level),
            menus::AUDIO_UNFOCUSED => audio.play_unfocused = !audio.play_unfocused,
            _ => return,
        }
        sfx.write(PlaySfx(Sfx::MenuMove));
        return;
    }
    let delta = delta as isize;
    match menus.options.selected() {
        // locked during a game
        menus::OPTION_MODE if options_from.in_game() => return,
        menus::OPTION_MODE => *prefs.mode = style::cycle(&Mode::ALL, *prefs.mode, delta),
        menus::OPTION_BACKGROUND => prefs.background.cycle(delta),
        menus::OPTION_BLOCKS => {
            prefs.style.block = style::cycle(&BlockStyle::ALL, prefs.style.block, delta)
        }
        menus::OPTION_BOARD => {
            prefs.style.board = style::cycle(&BoardStyle::ALL, prefs.style.board, delta)
        }
        menus::OPTION_AUDIO if delta > 0 => {
            open_audio(menus, sfx);
            return;
        }
        _ => return,
    }
    sfx.write(PlaySfx(Sfx::MenuMove));
}

fn name_entry(
    keys: Res<Keys>,
    mut initials: ResMut<Initials>,
    mut scores: ResMut<ScoreBoard>,
    mut settings: ResMut<crate::settings::Settings>,
    mut score_view: ResMut<ScoreView>,
    score: Res<Score>,
    no_save: Option<Res<NoSave>>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
) {
    for press in &keys.pressed {
        let key = press.key;
        if (is_enter(key) || key == KeyCode::Escape) && !press.repeat {
            // record the score with the entered initials
            let text = initials.text();
            score_view.highlight = scores.insert(HighScore {
                initials: text.clone(),
                score: score.score,
                level: score.level,
                lines: score.lines,
            });
            settings.initials = text;
            if no_save.is_none() {
                scores.save();
                settings.save();
            }
            sfx.write(PlaySfx(Sfx::MenuSelect));
            next.set(Screen::HighScores);
            return;
        }
        let moved = match key {
            KeyCode::Backspace => {
                initials.backspace();
                true
            }
            KeyCode::ArrowUp => {
                initials.step(1);
                true
            }
            KeyCode::ArrowDown => {
                initials.step(-1);
                true
            }
            KeyCode::ArrowLeft => {
                initials.move_cursor(-1);
                true
            }
            KeyCode::ArrowRight => {
                initials.move_cursor(1);
                true
            }
            _ => match press.text.filter(|c| c.is_ascii_alphabetic()) {
                Some(letter) => {
                    initials.type_letter(letter);
                    true
                }
                None => false,
            },
        };
        if moved {
            sfx.write(PlaySfx(Sfx::MenuMove));
        }
    }
}

/// any key leaves the score board, for a new game after one ends
fn high_scores(
    keys: Res<Keys>,
    mut score_view: ResMut<ScoreView>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
) {
    if keys.first().next().is_none() {
        return;
    }
    sfx.write(PlaySfx(Sfx::MenuSelect));
    score_view.highlight = None;
    score_view.from_game = false;
    // entering the menu starts a new game
    next.set(Screen::Menu);
}
