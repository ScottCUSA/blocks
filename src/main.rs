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

fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: VIEW_DIMENSIONS[0],
        window_height: VIEW_DIMENSIONS[1],
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    // initialize the debug logger
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Rustris");

    // find our assets path
    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder(ASSETS_FOLDER)
        .expect("unable to find assets folder");

    // load the font
    let font_path = assets_path.join("04b30.ttf");
    let font = load_ttf_font(&font_path.to_string_lossy())
        .await
        .expect("unable to load UI font");

    // load the background

    // let background1_path = assets_path.join("background1.wav");
    // let background1 = load_sound(&background1_path.to_string_lossy())
    //     .await
    //     .expect("unable to load background music");

    let background2_path = assets_path.join("background2.wav");
    let background2 = load_sound(&background2_path.to_string_lossy())
        .await
        .expect("unable to load background music");

    // setup parameters for drawing text
    let font_22pt = TextParams {
        font,
        font_size: 22,
        ..Default::default()
    };

    // setup parameters for drawing text
    let font_30pt = TextParams {
        font,
        font_size: 30,
        ..Default::default()
    };

    //
    let mut game = game::RustrisGame::new(board::RustrisBoard::new());

    let mut controls = controls::ControlStates::default();

    play_sound(
        background2,
        PlaySoundParams {
            looped: true,
            volume: 0.5,
        },
    );

    loop {
        clear_background(view::BACKGROUND_COLOR);

        // draw_text_ex(&get_fps().to_string(), 100., 200., overlay_text_params);
        game.update(&mut controls);
        game.draw(&font_22pt, &font_30pt);

        next_frame().await
    }
}
