use ::rand::seq::SliceRandom;
use ggez::{glam::IVec2, graphics::Color};
use strum::{EnumIter, EnumString, IntoEnumIterator};

// clockwise wall kicks from N, E, S and W, counterclockwise kicks undo the clockwise ones (guideline)
const JLSTZ_KICKS: [[[i32; 2]; 5]; 4] = [
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
];
const I_KICKS: [[[i32; 2]; 5]; 4] = [
    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
];

#[derive(Debug, Clone)]
pub struct Rustomino {
    pub rtype: RustominoType,
    pub state: RustominoState,
    turns: usize, // quarter turns clockwise from spawn
    pub blocks: [IVec2; 4],
    pub translation: IVec2,
}

impl Rustomino {
    pub fn new(rtype: RustominoType) -> Rustomino {
        let blocks = match rtype {
            RustominoType::I => [[0, 2], [1, 2], [2, 2], [3, 2]],
            RustominoType::O => [[1, 2], [2, 2], [2, 1], [1, 1]],
            RustominoType::T => [[1, 1], [0, 1], [1, 2], [2, 1]],
            RustominoType::L => [[1, 1], [0, 1], [2, 2], [2, 1]],
            RustominoType::J => [[1, 1], [0, 1], [0, 2], [2, 1]],
            RustominoType::S => [[1, 1], [0, 1], [1, 2], [2, 2]],
            RustominoType::Z => [[1, 1], [0, 2], [1, 2], [2, 1]],
        };
        let start_y = if rtype == RustominoType::I { 18 } else { 19 };
        Rustomino {
            rtype,
            state: RustominoState::Falling { time: 0. },
            turns: 0,
            blocks: blocks.map(IVec2::from),
            translation: IVec2::new(3, start_y),
        }
    }

    pub fn reset(self) -> Rustomino {
        Rustomino::new(self.rtype)
    }

    pub fn translate(&mut self, delta: IVec2) {
        self.translation += delta;
    }

    pub fn translated(&self, delta: &IVec2) -> [IVec2; 4] {
        translated(&self.blocks, &(self.translation + *delta))
    }

    pub fn playfield_slots(&self) -> [IVec2; 4] {
        translated(&self.blocks, &self.translation)
    }

    /// blocks a quarter turn about the pivot, in doubled units so the I and O stay whole
    fn turned(&self, rotation: &Rotation) -> [IVec2; 4] {
        let pivot = match self.rtype {
            RustominoType::I | RustominoType::O => IVec2::new(3, 3),
            _ => IVec2::new(2, 2),
        };
        self.blocks.map(|block| {
            let arm = (block * 2 - pivot).perp();
            let arm = match rotation {
                Rotation::Cw => -arm,
                Rotation::Ccw => arm,
            };
            (pivot + arm) / 2
        })
    }

    /// the kick moves the position, not the shape
    pub fn rotate(&mut self, rotation: &Rotation, kick: &IVec2) {
        self.blocks = self.turned(rotation);
        self.translation += *kick;
        self.turns = (self.turns + rotation.quarters()) % 4;
    }

    pub fn rotated(&self, rotation: &Rotation) -> [IVec2; 4] {
        translated(&self.turned(rotation), &self.translation)
    }

    pub fn wall_kick_tests(&self, rotation: &Rotation) -> [IVec2; 5] {
        let kicks = if self.rtype == RustominoType::I {
            &I_KICKS
        } else {
            &JLSTZ_KICKS
        };
        match rotation {
            Rotation::Cw => kicks[self.turns].map(IVec2::from),
            Rotation::Ccw => kicks[(self.turns + 3) % 4].map(|kick| -IVec2::from(kick)),
        }
    }
}

pub fn translated(blocks: &[IVec2; 4], delta: &IVec2) -> [IVec2; 4] {
    blocks.map(|block| block + *delta)
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, PartialEq, Eq)]
pub enum RustominoType {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

impl RustominoType {
    // piece colors in I O T L J S Z order
    const PALETTE: [Color; 7] = [
        Color::new(1.0, 0.45, 0.38, 1.0),  // coral
        Color::new(0.95, 0.35, 0.72, 1.0), // rose
        Color::new(1.0, 0.8, 0.25, 1.0),   // gold
        Color::new(0.62, 0.48, 1.0, 1.0),  // violet
        Color::new(0.9, 0.84, 0.7, 1.0),   // sand
        Color::new(0.32, 0.64, 1.0, 1.0),  // sky
        Color::new(0.15, 0.85, 0.55, 1.0), // emerald
    ];

    pub fn color(&self) -> Color {
        RustominoType::PALETTE[*self as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RustominoState {
    Falling { time: f64 },
    Lockdown { time: f64 },
}

#[derive(Debug, Clone)]
pub enum Rotation {
    Cw,
    Ccw,
}

impl Rotation {
    fn quarters(&self) -> usize {
        match self {
            Rotation::Cw => 1,
            Rotation::Ccw => 3,
        }
    }
}

pub struct RustominoBag {
    bag: Vec<RustominoType>, // contains the next rustomino types, shuffled
    rng: rand_xoshiro::Xoshiro256PlusPlus,
}

impl RustominoBag {
    pub fn new() -> Self {
        RustominoBag {
            bag: Vec::new(),
            rng: rand::make_rng(),
        }
    }

    pub fn get_next(&mut self) -> Rustomino {
        // one of each type, shuffled
        if self.bag.is_empty() {
            self.bag = RustominoType::iter().collect();
            self.bag.shuffle(&mut self.rng);
            log::debug!("filled rustomino bag: {:?}", self.bag);
        }
        let rtype = self.bag.pop().expect("rustomino bag is empty");
        log::info!("next rustomino type: {:?}", rtype);
        Rustomino::new(rtype)
    }
}
