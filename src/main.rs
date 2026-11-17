#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
// bevy systems take their data as parameters
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use std::{env, fmt, fs, path};

mod app;
mod audio;
mod background;
mod controls;
mod draw;
mod effects;
mod game;
mod gfx;
mod level_progress;
mod menus;
mod playfield;
mod render;
mod rustomino;
mod score_display;
mod scores;
mod settings;
mod style;
mod util;

const ASSETS_FOLDER: &str = "./resources";

#[derive(Debug)]
pub struct GameError(pub String);

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GameError {}

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> Self {
        GameError(e.to_string())
    }
}

pub type GameResult<T = ()> = Result<T, GameError>;

/// folders the game's files are read from, first match wins
#[derive(bevy::prelude::Resource)]
pub struct Resources {
    pub dirs: Vec<path::PathBuf>,
}

impl Resources {
    pub fn load(&self, file: &str) -> GameResult<Vec<u8>> {
        for dir in &self.dirs {
            match fs::read(dir.join(file)) {
                Ok(data) => return Ok(data),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(GameError(format!("could not read {file}: {e}"))),
            }
        }
        Err(GameError(format!("{file} not found in {:?}", self.dirs)))
    }
}

fn main() {
    // setup logging
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .filter(Some("wgpu_core::device"), log::LevelFilter::Warn)
        .filter(Some("wgpu_hal::vulkan::instance"), log::LevelFilter::Warn)
        .init();

    // setup resource path
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push(ASSETS_FOLDER);
        path
    } else {
        path::PathBuf::from(ASSETS_FOLDER)
    };
    log::info!("resource_dir: {:?}", resource_dir);
    let mut resources = Resources {
        dirs: vec![resource_dir],
    };
    // packages keep resources next to the executable
    if let Some(dir) = env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("resources")))
    {
        resources.dirs.push(dir);
    }
    if let Some(dir) = bundle_resource_dir() {
        resources.dirs.push(dir);
    }

    // --screenshot <scene> [path] renders one frame in a hidden window and exits
    let args: Vec<String> = env::args().collect();
    let screenshot = game::Screenshot::from_args(&args).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    // --bench [filter] [seconds per case] times the viewer's scenes and exits
    let bench = args.iter().position(|a| a == "--bench").map(|i| {
        let mut bench = game::bench::StartBench {
            hold: game::bench::DEFAULT_HOLD,
            filter: None,
            exit: true,
        };
        for arg in args[i + 1..].iter().take_while(|a| !a.starts_with("--")) {
            match arg.parse() {
                Ok(hold) => bench.hold = hold,
                Err(_) => bench.filter = Some(arg.clone()),
            }
        }
        bench
    });

    let settings = if screenshot.is_none() {
        settings::Settings::load()
    } else {
        settings::Settings::default()
    };

    app::run(resources, screenshot, bench, settings);
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
