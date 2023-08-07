use ggez::graphics;
use once_cell::sync::Lazy;

static MENU_ENTRIES: Lazy<Vec<String>> = Lazy::new(|| {
    let entries = vec![
        "Start Game".to_string(),
        "Options".to_string(),
        "Quit Game".to_string(),
    ];
    entries
});

static PAUSED_ENTRIES: Lazy<Vec<String>> = Lazy::new(|| {
    let entries = vec![
        "Resume Game".to_string(),
        "Options".to_string(),
        "Exit to Menu".to_string(),
        "Quit Game".to_string(),
    ];
    entries
});

pub(crate) trait Menu {
    fn items(&self) -> &Vec<graphics::Text>;
    fn selected(&self) -> usize;
    fn next(&mut self);
    fn previous(&mut self);
    fn reset_selection(&mut self);
    fn set_selection(&mut self, index: usize);
}

pub(crate) struct MenuState {
    menu: Vec<graphics::Text>,
    selected: usize,
}

impl MenuState {
    pub(crate) fn new() -> Self {
        let menu = MENU_ENTRIES
            .iter()
            .map(graphics::Text::new)
            .collect::<Vec<graphics::Text>>();
        MenuState { menu, selected: 0 }
    }
}

impl Menu for MenuState {
    fn items(&self) -> &Vec<graphics::Text> {
        &self.menu
    }
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.menu.len();
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.menu.len() - 1
        } else {
            self.selected - 1
        };
    }
    fn reset_selection(&mut self) {
        self.selected = 0;
    }
    fn set_selection(&mut self, index: usize) {
        assert!(index < self.menu.len());
        self.selected = index;
    }
}

pub(crate) struct PausedState {
    menu: Vec<graphics::Text>,
    selected: usize,
}

impl PausedState {
    pub(crate) fn new() -> Self {
        let menu = PAUSED_ENTRIES
            .iter()
            .map(graphics::Text::new)
            .collect::<Vec<graphics::Text>>();
        PausedState { menu, selected: 0 }
    }
}

impl Menu for PausedState {
    fn items(&self) -> &Vec<graphics::Text> {
        &self.menu
    }
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.menu.len();
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.menu.len() - 1
        } else {
            self.selected - 1
        };
    }
    fn reset_selection(&mut self) {
        self.selected = 0;
    }
    fn set_selection(&mut self, index: usize) {
        assert!(index < self.menu.len());
        self.selected = index;
    }
}
