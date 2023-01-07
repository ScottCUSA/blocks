// #![windows_subsystem = "windows"]
use macroquad::{
    prelude::*, text::load_ttf_font, window::Conf,
};

mod board;
mod game;
mod rustomino;
mod view;
mod controls;

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
    let font = load_ttf_font(&font_path.to_string_lossy()).await.expect("unable to load UI font");

    // setup parameters for drawing text
    let overlay_text_params = TextParams {
        font,
        font_size: 22,
        ..Default::default()
    };

    // 
    let mut game = game::RustrisGame::new(
        board::RustrisBoard::new(),
        view::ViewSettings::new(VIEW_DIMENSIONS),
    );

    let mut controls = controls::ControlStates::default();

    loop {
        clear_background(view::BACKGROUND_COLOR);

        game.update(&mut controls);
        game.draw(&overlay_text_params);

        next_frame().await
    }
}
