use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{draw, game};

const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    pub width: f32, // logical pixels
    pub height: f32,
    pub x: Option<i32>, // physical pixels, outer position
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: draw::VIEW_WIDTH,
            height: draw::VIEW_HEIGHT,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// options and window state saved between runs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: String,
    pub theme: String,
    pub background: String,
    pub blocks: String,
    pub board: String,
    pub music_level: u8,
    pub sound_level: u8,
    pub play_unfocused: bool, // keep the music playing when the window loses focus
    pub menu_sounds: bool,
    pub initials: String,
    pub window: WindowState,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: String::new(),
            theme: String::new(),
            background: String::new(),
            blocks: String::new(),
            board: String::new(),
            music_level: game::DEFAULT_VOLUME_LEVEL,
            sound_level: game::DEFAULT_VOLUME_LEVEL,
            play_unfocused: false,
            menu_sounds: true,
            initials: String::new(),
            window: WindowState::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        config_path(SETTINGS_FILE)
            .and_then(|path| load_toml(&path))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(path) = config_path(SETTINGS_FILE) {
            save_toml(&path, self);
        }
    }
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "Scott Cummings", "Blocks")
}

pub fn config_path(file: &str) -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().join(file))
}

pub fn data_path(file: &str) -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.data_dir().join(file))
}

/// None if missing or unreadable
pub fn load_toml<T: DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("could not read {path:?}: {e}");
            return None;
        }
    };
    toml::from_str(&text)
        .inspect_err(|e| log::warn!("could not parse {path:?}: {e}"))
        .ok()
}

/// write to a temp file then rename
pub fn save_toml<T: Serialize>(path: &PathBuf, value: &T) {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, toml::to_string(value)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    match result {
        Ok(()) => log::info!("saved {path:?}"),
        Err(e) => log::warn!("could not save {path:?}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let settings = Settings {
            mode: "Hard".into(),
            theme: "Bounce".into(),
            background: "Stars".into(),
            blocks: "Neon".into(),
            board: "Dots".into(),
            music_level: 3,
            sound_level: 7,
            play_unfocused: true,
            menu_sounds: false,
            initials: "ABC".into(),
            window: WindowState {
                width: 1280.0,
                height: 960.0,
                x: Some(-20),
                y: Some(40),
                maximized: true,
            },
        };
        let text = toml::to_string(&settings).unwrap();
        assert_eq!(toml::from_str::<Settings>(&text).unwrap(), settings);
    }

    #[test]
    fn missing_fields_default() {
        let settings: Settings = toml::from_str("music_level = 2").unwrap();
        assert_eq!(settings.music_level, 2);
        assert_eq!(settings.sound_level, 5);
        // older settings files keep the music focus only
        assert!(!settings.play_unfocused);
        assert!(settings.menu_sounds);
        assert_eq!(settings.window, WindowState::default());
    }
}
