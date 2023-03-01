#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use ggez::{event, ContextBuilder};

mod controls;
mod game;
mod playfield;
mod rustomino;
mod view;

// TODO: load icon for rustris window
// https://docs.rs/macroquad/0.3.25/macroquad/texture/struct.Image.html

fn main() {
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    let (mut ctx, event_loop) = ContextBuilder::new("Rustris", "Scott Cummings")
        .build()
        .expect("could not create engine context");

    let playfield = playfield::RustrisPlayfield::new();
    let game = game::RustrisGame::new(playfield);

    event::run(ctx, event_loop, game);
}
