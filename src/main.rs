// #![windows_subsystem = "windows"]
use macroquad::{
    experimental::collections::storage, prelude::*, text::load_ttf_font, window::Conf,
};

mod board;
mod game;
mod rustomino;
mod view;

const WINDOW_DIMENSIONS: [i32; 2] = [1024, 768];
const ASSETS_FOLDER: &str = "assets";

fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: WINDOW_DIMENSIONS[0],
        window_height: WINDOW_DIMENSIONS[1],
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Rustris");

    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder(ASSETS_FOLDER)
        .expect("unable to find assets folder");

    let font_path = assets_path.join("04b30.ttf");
    let font = load_ttf_font(font_path.to_str().unwrap()).await.unwrap();

    let text_params = TextParams {
        font,
        font_size: 22,
        ..Default::default()
    };

    storage::store(font);

    let mut game = game::RustrisGame::new(
        board::RustrisBoard::new(),
        view::ViewSettings::new(WINDOW_DIMENSIONS),
    );

    let mut inputs = game::GameInputs::default();

    // scene::add_node(controller);

    loop {
        clear_background(view::BACKGROUND_COLOR);

        game.update(&mut inputs);
        game.draw();
        game.draw_overlay(&text_params);

        next_frame().await
    }
}
