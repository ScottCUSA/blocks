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
pub const AUDIO_MENU_SOUNDS: usize = 4;
pub const AUDIO_BACK: usize = 5;

pub struct MenuList {
    pub items: &'static [&'static str],
    pub selected: Option<usize>, // none while the mouse leads and is off the rows
}

impl MenuList {
    fn new(items: &'static [&'static str]) -> Self {
        Self {
            items,
            selected: Some(0),
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
        Self::new(&["Song", "Music", "Sound", "Unfocused", "Menu Sounds", "Back"])
    }

    /// down a row, wrapping, from none to the top
    pub fn next(&mut self) {
        let len = self.items.len();
        self.selected = Some(self.selected.map_or(0, |i| (i + 1) % len));
    }

    /// up a row, wrapping, from none to the top
    pub fn previous(&mut self) {
        let len = self.items.len();
        self.selected = Some(self.selected.map_or(0, |i| (i + len - 1) % len));
    }

    pub fn reset_selection(&mut self) {
        self.selected = Some(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_wrap_and_start_at_top() {
        let mut menu = MenuList::main();
        menu.previous();
        assert_eq!(menu.selected, Some(3));
        menu.next();
        assert_eq!(menu.selected, Some(0));
        for step in [MenuList::next, MenuList::previous] {
            menu.selected = None;
            step(&mut menu);
            assert_eq!(menu.selected, Some(0));
        }
    }
}
