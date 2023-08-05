#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use std::{env, path};

use ggez::{conf, event, ContextBuilder};

mod controls;
mod draw;
mod game;
mod playfield;
mod rustomino;
mod util;

const ASSETS_FOLDER: &str = "resources";

// TODO: load icon for rustris window
// https://docs.rs/macroquad/0.3.25/macroquad/texture/struct.Image.html

fn main() {
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .filter(Some("wgpu_core::device"), log::LevelFilter::Warn)
        .filter(Some("wgpu_hal::vulkan::instance"), log::LevelFilter::Warn)
        .init();

    log::info!("loading Resources");

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push(ASSETS_FOLDER);
        path
    } else {
        path::PathBuf::from(ASSETS_FOLDER)
    };
    log::info!("resource_dir: {:?}", resource_dir);

    let (mut ctx, event_loop) = ContextBuilder::new("Rustris", "Scott Cummings")
        .window_setup(conf::WindowSetup::default().title("Rustris"))
        .window_mode(
            conf::WindowMode::default()
                .resizable(false)
                .dimensions(1024., 768.),
        )
        .add_resource_path(resource_dir)
        .build()
        .expect("could not create engine context");

    let game = game::RustrisGame::new(&mut ctx).expect("unable to initialize gamestate");

    event::run(ctx, event_loop, game);
}
