#![windows_subsystem = "windows"]

use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
    text::load_ttf_font,
    window::Conf,
};

mod board;
mod controls;
mod game;
mod rustomino;
mod view;

const VIEW_DIMENSIONS: [i32; 2] = [1024, 768];
const ASSETS_FOLDER: &str = "assets";
const BACKGROUND_MUSIC_VOL: f32 = 0.1;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: VIEW_DIMENSIONS[0],
        window_height: VIEW_DIMENSIONS[1],
        window_resizable: false,
        ..Default::default()
    }
}

// TODO: implement move reset lock down
// https://tetris.wiki/Tetris_Guideline
// TODO: implement rotation wall kicks

#[macroquad::main(window_conf())]
async fn main() {
    // initialize the debug logger
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Rustris;");

    // find our assets path
    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder(ASSETS_FOLDER)
        .expect("unable to find assets folder");

    log::info!("Loading Resources");
    // load the font
    let font_path = assets_path.join("04b30.ttf");
    let font = load_ttf_font(&font_path.to_string_lossy())
        .await
        .expect("unable to load UI font");

    // setup parameters for drawing text
    let font_20pt = TextParams {
        font,
        font_size: 20,
        ..Default::default()
    };

    // setup parameters for drawing text
    let font_30pt = TextParams {
        font,
        font_size: 30,
        ..Default::default()
    };

    log::info!("Loading font: {:?}", font_path);

    let mut game = game::RustrisGame::new(board::RustrisBoard::new());
    let mut controls = controls::ControlStates::default();

    // load the background music

    // let background_path = assets_path.join("background1.wav");
    // let background = load_sound(&background_path.to_string_lossy())
    //     .await
    //     .expect("unable to load background music");

    let background_path = assets_path.join("background.ogg");
    log::info!("Loading background music: {:?}", background_path);
    let background = load_sound(&background_path.to_string_lossy())
        .await
        .expect("unable to load background music");

    //

    play_sound(
        background,
        PlaySoundParams {
            looped: true,
            volume: BACKGROUND_MUSIC_VOL,
        },
    );
    log::info!("Playing background music at volume: {BACKGROUND_MUSIC_VOL}");

    loop {
        clear_background(view::BACKGROUND_COLOR);

        // draw FPS
        // draw_text_ex(
        //     &get_fps().to_string(),
        //     VIEW_DIMENSIONS[0] as f32 - 100.,
        //     50.,
        //     font_22pt,
        // );
        game.update(background, &mut controls);
        game.draw(&font_20pt, &font_30pt);

        next_frame().await
    }
}
