// #![windows_subsystem = "windows"]
use crate::{board::RustrisBoard, game::RustrisGame};

mod board;
mod game;
mod rustomino;
mod view;

#[macroquad::main("Rustris")]
async fn main() {
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Rustris");

    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder("assets")
        .expect("unable to open assets path");

    // let font_path = assets_path.join("04b30.ttf");
    // let font = load_ttf_font(font_path.to_str().unwrap()).await.unwrap();
    let game = RustrisGame::new(RustrisBoard::new());

    // scene::add_node(controller);
    // let mut view = RustrisView::new(WINDOW_DIMENSIONS, &assets_path);
    // let mut controller = RustrisController::new(rustris_board).init();

    game.run().await;
}
