use crate::{board::RustrisBoard, controller::RustrisController, view::RustrisView};
use piston_window::{types::Color, *};

mod board;
mod controller;
mod rustomino;
mod view;

const WINDOW_DIMENSIONS: [u32; 2] = [1024, 768];

fn main() {
    env_logger::init_from_env("RUSTRIS_LOG_LEVEL");
    log::info!("Startup: Initializing Piston Window");
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Rustris", WINDOW_DIMENSIONS)
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .expect("fatal error, could not create window");

    let rustris_board = RustrisBoard::new();
    let mut rustris_controller = RustrisController::new(rustris_board).init();
    let mut rustris_view = RustrisView::new(WINDOW_DIMENSIONS);

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            rustris_controller.key_pressed(key);
        }
        if let Some(Button::Keyboard(key)) = event.release_args() {
            rustris_controller.key_released(key);
        }
        if let Some(args) = event.resize_args() {
            rustris_view.resize(args);
        }
        window.draw_2d(&event, |c, g, _| {
            rustris_view.draw(&rustris_controller, &c, g)
        });
        event.update(|arg| {
            rustris_controller.update(arg.dt);
        });
    }
}
