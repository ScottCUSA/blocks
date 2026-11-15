/// how blocks are drawn
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockStyle {
    Bevel,
    Flat,
    Neon,
    Retro,
    Jelly,
}

impl BlockStyle {
    pub const ALL: [BlockStyle; 5] = [
        BlockStyle::Bevel,
        BlockStyle::Flat,
        BlockStyle::Neon,
        BlockStyle::Retro,
        BlockStyle::Jelly,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BlockStyle::Bevel => "Bevel",
            BlockStyle::Flat => "Flat",
            BlockStyle::Neon => "Neon",
            BlockStyle::Retro => "Retro",
            BlockStyle::Jelly => "Jelly",
        }
    }
}

/// how the playfield is drawn
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoardStyle {
    Grid,
    Dots,
    Checker,
    Clean,
}

impl BoardStyle {
    pub const ALL: [BoardStyle; 4] = [
        BoardStyle::Grid,
        BoardStyle::Dots,
        BoardStyle::Checker,
        BoardStyle::Clean,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BoardStyle::Grid => "Grid",
            BoardStyle::Dots => "Dots",
            BoardStyle::Checker => "Checker",
            BoardStyle::Clean => "Clean",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub block: BlockStyle,
    pub board: BoardStyle,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            block: BlockStyle::Bevel,
            board: BoardStyle::Grid,
        }
    }
}

/// step through a list of styles, wrapping
pub fn cycle<T: Copy + PartialEq>(all: &[T], current: T, delta: isize) -> T {
    let i = all.iter().position(|s| *s == current).unwrap_or(0) as isize;
    all[(i + delta).rem_euclid(all.len() as isize) as usize]
}

/// style with the given name
pub fn from_name<T: Copy>(all: &[T], name: &str, name_of: fn(T) -> &'static str) -> Option<T> {
    all.iter().copied().find(|s| name_of(*s) == name)
}
