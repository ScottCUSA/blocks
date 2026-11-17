use crate::gfx as graphics;

// options menu rows
pub const OPTION_MODE: usize = 0;
pub const OPTION_BACKGROUND: usize = 1;
pub const OPTION_BLOCKS: usize = 2;
pub const OPTION_BOARD: usize = 3;
pub const OPTION_AUDIO: usize = 4;
pub const OPTION_BACK: usize = 5;

// audio submenu rows
pub const AUDIO_THEME: usize = 0;
pub const AUDIO_MUSIC: usize = 1;
pub const AUDIO_SOUND: usize = 2;
pub const AUDIO_UNFOCUSED: usize = 3;
pub const AUDIO_BACK: usize = 4;

pub trait Menu {
    fn items(&self) -> &Vec<graphics::Text>;
    fn selected(&self) -> usize;
    fn next(&mut self);
    fn previous(&mut self);
    fn reset_selection(&mut self);
}

pub struct MenuList {
    items: Vec<graphics::Text>,
    selected: usize,
}

impl MenuList {
    fn new(entries: &[&str]) -> Self {
        Self {
            items: entries.iter().map(|e| graphics::Text::new(*e)).collect(),
            selected: 0,
        }
    }

    pub fn main() -> Self {
        Self::new(&["Start Game", "Options", "High Scores", "Quit Game"])
    }

    pub fn paused() -> Self {
        Self::new(&["Resume Game", "Options", "Exit to Menu", "Quit Game"])
    }

    pub fn options() -> Self {
        Self::new(&["Mode", "Background", "Blocks", "Board", "Audio", "Back"])
    }

    pub fn audio() -> Self {
        Self::new(&["Song", "Music", "Sound", "Unfocused", "Back"])
    }
}

impl Menu for MenuList {
    fn items(&self) -> &Vec<graphics::Text> {
        &self.items
    }
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }
    fn reset_selection(&mut self) {
        self.selected = 0;
    }
}

/// every menu's selection
#[derive(bevy::prelude::Resource)]
pub struct Menus {
    pub main: MenuList,
    pub paused: MenuList,
    pub options: MenuList,
    pub audio: MenuList,
    pub audio_open: bool, // the audio page of the options
}

impl Default for Menus {
    fn default() -> Self {
        Menus {
            main: MenuList::main(),
            paused: MenuList::paused(),
            options: MenuList::options(),
            audio: MenuList::audio(),
            audio_open: false,
        }
    }
}
