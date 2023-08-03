#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use view::window_conf;

mod controls;
mod game;
mod playfield;
mod rustomino;
mod view;

// TODO: load icon for rustris window
// https://docs.rs/macroquad/0.3.25/macroquad/texture/struct.Image.html

#[macroquad::main(window_conf())]
async fn main() {
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    // macroquad isn't scaling the window at startup
    // correctly, this fixes this it at runtime
    macroquad::window::request_new_screen_size(view::VIEW_WH[0] as f32, view::VIEW_WH[1] as f32);
    game::run().await
}
