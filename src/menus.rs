use ggez::graphics;

// options menu rows
pub const OPTION_MODE: usize = 0;
pub const OPTION_THEME: usize = 1;
pub const OPTION_BACKGROUND: usize = 2;
pub const OPTION_BLOCKS: usize = 3;
pub const OPTION_BOARD: usize = 4;
pub const OPTION_MUSIC: usize = 5;
pub const OPTION_SOUND: usize = 6;
pub const OPTION_BACK: usize = 7;

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
        Self::new(&[
            "Mode",
            "Song",
            "Background",
            "Blocks",
            "Board",
            "Music",
            "Sound",
            "Back",
        ])
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
