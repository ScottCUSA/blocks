// #![windows_subsystem = "windows"]
use crate::{board::RustrisBoard, controller::RustrisController, view::RustrisView};
use opengl_graphics::{GlGraphics, OpenGL};
use piston_window::Size;

mod board;
mod controller;
mod rustomino;
mod view;

const WINDOW_DIMENSIONS: Size = Size {
    width: 1024.0,
    height: 768.0,
};

fn main() {
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Piston Window");
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Rustris", WINDOW_DIMENSIONS)
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .expect("fatal error, could not create window");

    let opengl = OpenGL::V4_5;
    let mut gl = GlGraphics::new(opengl);

    let assets_path = find_folder::Search::ParentsThenKids(2, 2)
        .for_folder("assets")
        .expect("unable to open assets path");

    let rustris_board = RustrisBoard::new();
    let mut rustris_view = RustrisView::new(WINDOW_DIMENSIONS, &assets_path);
    let mut rustris_controller = RustrisController::new(rustris_board).init();

    rustris_controller.run(&mut window, &mut gl, &mut rustris_view);
}
