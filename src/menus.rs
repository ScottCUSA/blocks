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

pub struct MenuList {
    pub items: &'static [&'static str],
    selected: usize,
}

impl MenuList {
    fn new(items: &'static [&'static str]) -> Self {
        Self { items, selected: 0 }
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

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        self.selected = (self.selected + self.items.len() - 1) % self.items.len();
    }

    pub fn reset_selection(&mut self) {
        self.selected = 0;
    }
}
