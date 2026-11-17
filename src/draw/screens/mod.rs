mod debug;
mod demo;
mod game_over;
mod high_scores;
mod menu;
mod name_entry;
mod options;
mod paused;

pub use debug::{DebugScreen, FrameStats};
pub use demo::DemoScreen;
pub use game_over::GameOverScreen;
pub use high_scores::HighScoresScreen;
pub use menu::MenuScreen;
pub use name_entry::NameEntryScreen;
pub use options::{OptionRow, OptionsScreen};
pub use paused::PausedScreen;

pub(super) const TITLE_SIZE: f32 = 96.0;
pub(super) const MENU_PANEL_WIDTH: f32 = 520.0;
pub(super) const PAUSED_TITLE_SIZE: f32 = 64.0;
