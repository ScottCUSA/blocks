#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use view::window_conf;

mod controller;
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

    controller::run().await
}
