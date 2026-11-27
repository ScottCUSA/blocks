use strum::{IntoStaticStr, VariantArray};

/// one of a fixed list of options, saved by name
pub trait Choice: Copy + PartialEq + VariantArray + Into<&'static str> {
    fn name(self) -> &'static str {
        self.into()
    }

    /// step through the list, wrapping
    fn cycle(self, delta: isize) -> Self {
        let all = Self::VARIANTS;
        let i = all.iter().position(|s| *s == self).unwrap_or(0) as isize;
        all[(i + delta).rem_euclid(all.len() as isize) as usize]
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::VARIANTS.iter().copied().find(|s| s.name() == name)
    }
}

impl<T: Copy + PartialEq + VariantArray + Into<&'static str>> Choice for T {}

/// how blocks are drawn
#[derive(Debug, Clone, Copy, PartialEq, Default, VariantArray, IntoStaticStr)]
pub enum BlockStyle {
    #[default]
    Bevel,
    Flat,
    Neon,
    Retro,
    Jelly,
}

/// how the playfield is drawn
#[derive(Debug, Clone, Copy, PartialEq, Default, VariantArray, IntoStaticStr)]
pub enum BoardStyle {
    #[default]
    Grid,
    Dots,
    Checker,
    Clean,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub block: BlockStyle,
    pub board: BoardStyle,
}
