use ggez::graphics;
use once_cell::sync::Lazy;

static MENU_ENTRIES: Lazy<Vec<String>> = Lazy::new(|| {
    let entries = vec![
        "Start Game".to_string(),
        "Controls".to_string(),
        "Quit".to_string(),
    ];
    entries
});

static PAUSED_ENTRIES: Lazy<Vec<String>> = Lazy::new(|| {
    let entries = vec![
        "Resume Game".to_string(),
        "Controls".to_string(),
        "Quit".to_string(),
    ];
    entries
});

pub struct MenuState {
    pub menu: Vec<graphics::Text>,
    pub selected: usize,
}

impl MenuState {
    pub fn new() -> Self {
        let menu = MENU_ENTRIES
            .iter()
            .map(graphics::Text::new)
            .collect::<Vec<graphics::Text>>();
        let selected = 0;
        MenuState { menu, selected }
    }
}

pub struct PausedState {
    pub menu: Vec<graphics::Text>,
    pub selected: usize,
}

impl PausedState {
    pub fn new() -> Self {
        let menu = PAUSED_ENTRIES
            .iter()
            .map(graphics::Text::new)
            .collect::<Vec<graphics::Text>>();
        let selected = 0;
        PausedState { menu, selected }
    }
}
