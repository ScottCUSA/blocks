#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use std::{env, path};

use ggez::{conf, event, ContextBuilder};

use game::bench;

mod background;
mod controls;
mod draw;
mod effects;
mod game;
mod level_progress;
mod menus;
mod playfield;
mod rustomino;
mod score_display;
mod scores;
mod settings;
mod style;
mod util;

const ASSETS_FOLDER: &str = "./resources";

// TODO: load icon for blocks window
// https://docs.rs/macroquad/0.3.25/macroquad/texture/struct.Image.html

fn main() {
    // setup logging
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .filter(Some("wgpu_core::device"), log::LevelFilter::Warn)
        .filter(Some("wgpu_hal::vulkan::instance"), log::LevelFilter::Warn)
        .init();

    // setup resource path
    let resource_dir =
        path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default()).join(ASSETS_FOLDER);
    log::info!("resource_dir: {:?}", resource_dir);

    // --screenshot <scene> [path] renders one frame in a hidden window and exits
    let args: Vec<String> = env::args().collect();
    let screenshot = game::Screenshot::from_args(&args).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    // --bench [filter] [seconds per case] times the viewer's scenes and exits
    let bench = args.iter().position(|a| a == "--bench").map(|i| {
        let mut request = bench::StartBench {
            hold: bench::DEFAULT_HOLD,
            filter: None,
            exit: true,
            vsync: false,
        };
        for arg in args[i + 1..].iter().take_while(|a| !a.starts_with("--")) {
            match arg.parse() {
                Ok(hold) => request.hold = hold,
                Err(_) => request.filter = Some(arg.clone()),
            }
        }
        request
    });

    // screenshots don't use or change the player's settings
    let settings = screenshot
        .as_ref()
        .map_or_else(settings::Settings::load, |_| Default::default());

    // setup game context
    let mut builder = ContextBuilder::new("Blocks", "Scott Cummings")
        .window_setup(
            conf::WindowSetup::default()
                .title("Blocks")
                // a benchmark runs uncapped
                .vsync(bench.is_none()),
        )
        .window_mode(game::window_mode(screenshot.is_some(), &settings.window))
        .add_resource_path(resource_dir);
    if let Some(dir) = bundle_resource_dir() {
        builder = builder.add_resource_path(dir);
    }
    let (mut ctx, event_loop) = builder.build().expect("could not create engine context");

    if screenshot.is_none() {
        game::show_window(&ctx, &settings.window);
    }

    // setup game state
    let game = game::BlocksState::new(&mut ctx, screenshot, bench, settings)
        .expect("unable to initialize gamestate");

    // run game
    event::run(ctx, event_loop, game).expect("game loop error");
}

// macOS .app bundle: Contents/MacOS/blocks -> Contents/Resources
fn bundle_resource_dir() -> Option<path::PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let dir = env::current_exe()
        .ok()?
        .parent()?
        .parent()?
        .join("Resources");
    dir.join("04b30.ttf").exists().then_some(dir)
}
