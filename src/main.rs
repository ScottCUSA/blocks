use macroquad::window::{clear_background, next_frame, Conf};

// #![windows_subsystem = "windows"]

mod board;
mod game;
mod rustomino;
mod view;

const WINDOW_DIMENSIONS: [f32; 2] = [1024., 768.];

fn window_conf() -> Conf {
    Conf {
        window_title: "Rustris".to_owned(),
        window_width: WINDOW_DIMENSIONS[0] as i32,
        window_height: WINDOW_DIMENSIONS[1] as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Rustris");

    // let assets_path = find_folder::Search::ParentsThenKids(2, 2)
    //     .for_folder("assets")
    //     .expect("unable to open assets path");

    // let font_path = assets_path.join("04b30.ttf");
    // let font = load_ttf_font(font_path.to_str().unwrap()).await.unwrap();

    let mut game = game::RustrisGame::new(
        board::RustrisBoard::new(),
        view::ViewSettings::new(WINDOW_DIMENSIONS),
    );

    // scene::add_node(controller);

    loop {
        clear_background(view::BACKGROUND_COLOR);

        game.update();
        game.draw();

        next_frame().await
    }
}
