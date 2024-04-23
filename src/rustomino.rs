use ::rand::{seq::SliceRandom, SeedableRng};
use ggez::{glam::IVec2, graphics::Color};
use strum::{EnumIter, IntoEnumIterator};

const I_START_TRANSLATION: IVec2 = IVec2::new(3, 18);
const O_T_L_J_S_Z_START_TRANSLATION: IVec2 = IVec2::new(3, 19);

const I_BLOCKS: [IVec2; 4] = [
    IVec2::new(0, 2),
    IVec2::new(1, 2),
    IVec2::new(2, 2),
    IVec2::new(3, 2),
];
const O_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 2),
    IVec2::new(2, 2),
    IVec2::new(2, 1),
    IVec2::new(1, 1),
];
const T_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 2),
    IVec2::new(2, 1),
];
const L_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(2, 2),
    IVec2::new(2, 1),
];
const J_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(0, 2),
    IVec2::new(2, 1),
];
const S_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 2),
    IVec2::new(2, 2),
];
const Z_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 2),
    IVec2::new(1, 2),
    IVec2::new(2, 1),
];

const I_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(2, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, -2),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(1, -2),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(-2, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(-2, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 2),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(-1, 2),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(2, -1),
    ],
];

const O_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
    ],
];

const T_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
    ],
];

const L_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-2, 0),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(2, 0),
        IVec2::new(1, -1),
    ],
];

const J_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(2, 0),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(0, -2),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-2, 0),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, 2),
        IVec2::new(1, -1),
    ],
];

const S_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(0, -2),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
        IVec2::new(-2, 0),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
        IVec2::new(0, 2),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
        IVec2::new(2, 0),
    ],
];

const Z_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(0, -2),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(0, 2),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
    ],
];

const JLSTZ_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E (0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -2),
    ],
    [
        // E->N (0, 0),(1, 0),(1,-1),( 0,2),(1,2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 2),
    ],
    [
        // E->S (0, 0),(1, 0),(1,-1),(0, 2),(1, 2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 2),
    ],
    [
        // S->E ( 0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -2),
    ],
    [
        // E->W ( 0, 0),(1, 0),(1,1),( 0,-2),(1,-2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(1, -2),
    ],
    [
        // W->E ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(-1, 2),
    ],
    [
        // W->N ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(-1, 2),
    ],
    [
        // N->W (0, 0),(1, 0),(1, 1),(0, -2),(1, -2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(1, -2),
    ],
];

const I_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, -1),
        IVec2::new(1, 2),
    ],
    [
        // E->N ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 1),
        IVec2::new(-1, -2),
    ],
    [
        // E->S ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 2),
        IVec2::new(2, -1),
    ],
    [
        // S->E ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, -2),
        IVec2::new(-2, 1),
    ],
    [
        // E->W ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 1),
        IVec2::new(-1, -2),
    ],
    [
        // W->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, -1),
        IVec2::new(1, 2),
    ],
    [
        // W->N ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, -2),
        IVec2::new(-2, 1),
    ],
    [
        // N->W ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 2),
        IVec2::new(2, -1),
    ],
];

#[derive(Debug, Clone)]
pub struct Rustomino {
    pub rtype: RustominoType,
    pub state: RustominoState,
    pub rotation: RustominoRotation,
    pub blocks: [IVec2; 4],
    pub translation: IVec2,
}

impl Rustomino {
    pub fn new(rtype: RustominoType) -> Rustomino {
        let (rotation, blocks, translation) = {
            match rtype {
                RustominoType::I => (
                    RustominoRotation::new(I_ROTATIONS),
                    I_BLOCKS,
                    I_START_TRANSLATION,
                ),
                RustominoType::O => (
                    RustominoRotation::new(O_ROTATIONS),
                    O_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::T => (
                    RustominoRotation::new(T_ROTATIONS),
                    T_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::L => (
                    RustominoRotation::new(L_ROTATIONS),
                    L_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::J => (
                    RustominoRotation::new(J_ROTATIONS),
                    J_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::S => (
                    RustominoRotation::new(S_ROTATIONS),
                    S_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::Z => (
                    RustominoRotation::new(Z_ROTATIONS),
                    Z_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
            }
        };
        Rustomino {
            rtype,
            state: RustominoState::Falling { time: 0. },
            rotation,
            blocks,
            translation,
        }
    }

    pub fn reset(self) -> Rustomino {
        Rustomino::new(self.rtype)
    }

    pub fn translate(&mut self, delta: IVec2) {
        self.translation += delta;
    }

    pub fn translated(&self, delta: &IVec2) -> [IVec2; 4] {
        translated(&translated(&self.blocks, &self.translation), delta)
    }

    pub fn playfield_slots(&self) -> [IVec2; 4] {
        self.translated(&IVec2::ZERO)
    }

    pub fn rotate(&mut self, rotation: &Rotation, translation: &IVec2) {
        let rotation_trans = self.rotation.get_rotation_trans(rotation);

        for (i, item) in rotation_trans.iter().enumerate() {
            self.blocks[i] = self.blocks[i] + *item + *translation;
        }

        self.rotation.rotate(rotation);
    }

    pub fn rotated(&self, rotation: &Rotation) -> [IVec2; 4] {
        let rotation = self.rotation.get_rotation_trans(rotation);
        let mut result = [IVec2::ZERO; 4];

        for i in 0..4 {
            result[i] = self.blocks[i] + self.translation + rotation[i];
        }

        result
    }

    pub fn wall_kick_tests(&self, rotation: &Rotation) -> [IVec2; 5] {
        self.rotation.get_wall_kick_tests(self.rtype, rotation)
    }

    pub fn set_state(&mut self, state: RustominoState) {
        log::trace!("setting rustomino state: {:?}", state);
        self.state = state;
    }
}

pub fn translated(blocks: &[IVec2; 4], delta: &IVec2) -> [IVec2; 4] {
    [
        blocks[0] + *delta,
        blocks[1] + *delta,
        blocks[2] + *delta,
        blocks[3] + *delta,
    ]
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
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
    const CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0);
    const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
    const PURPLE: Color = Color::new(0.72, 0.01, 0.99, 1.0);
    const ORANGE: Color = Color::new(1.0, 0.45, 0.03, 1.0);
    const BLUE: Color = Color::new(0.09, 0.2, 1.0, 1.0);
    const GREEN: Color = Color::new(0.4, 0.99, 0.0, 1.0);
    const RED: Color = Color::new(1.0, 0.06, 0.24, 1.0);

    pub fn color(&self) -> Color {
        match self {
            RustominoType::I => RustominoType::CYAN,
            RustominoType::O => RustominoType::YELLOW,
            RustominoType::T => RustominoType::PURPLE,
            RustominoType::L => RustominoType::ORANGE,
            RustominoType::J => RustominoType::BLUE,
            RustominoType::S => RustominoType::GREEN,
            RustominoType::Z => RustominoType::RED,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RustominoState {
    Falling { time: f64 },
    Lockdown { time: f64 },
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn rotate(&self, rotation: &Rotation) -> Direction {
        match self {
            Direction::N => match rotation {
                Rotation::Cw => Direction::E,
                Rotation::Ccw => Direction::W,
            },
            Direction::E => match rotation {
                Rotation::Cw => Direction::S,
                Rotation::Ccw => Direction::N,
            },
            Direction::S => match rotation {
                Rotation::Cw => Direction::W,
                Rotation::Ccw => Direction::E,
            },
            Direction::W => match rotation {
                Rotation::Cw => Direction::N,
                Rotation::Ccw => Direction::S,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Rotation {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy)]
pub struct RustominoRotation {
    direction: Direction,
    n2e_trans: [IVec2; 4],
    e2s_trans: [IVec2; 4],
    s2w_trans: [IVec2; 4],
    w2n_trans: [IVec2; 4],
}

impl RustominoRotation {
    fn new(values: [[IVec2; 4]; 4]) -> Self {
        Self {
            direction: Direction::N,
            n2e_trans: values[0],
            e2s_trans: values[1],
            s2w_trans: values[2],
            w2n_trans: values[3],
        }
    }

    fn get_rotation_trans(&self, rotation: &Rotation) -> [IVec2; 4] {
        match self.direction {
            Direction::N => match rotation {
                Rotation::Cw => self.n2e_trans,
                Rotation::Ccw => neg_trans(self.w2n_trans),
            },
            Direction::E => match rotation {
                Rotation::Cw => self.e2s_trans,
                Rotation::Ccw => neg_trans(self.n2e_trans),
            },
            Direction::S => match rotation {
                Rotation::Cw => self.s2w_trans,
                Rotation::Ccw => neg_trans(self.e2s_trans),
            },
            Direction::W => match rotation {
                Rotation::Cw => self.w2n_trans,
                Rotation::Ccw => neg_trans(self.s2w_trans),
            },
        }
    }

    fn get_wall_kick_tests(&self, rtype: RustominoType, rotation: &Rotation) -> [IVec2; 5] {
        match self.direction {
            Direction::N => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[0],
                    _ => JLSTZ_WALL_KICK_TESTS[0],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[7],
                    _ => JLSTZ_WALL_KICK_TESTS[7],
                },
            },
            Direction::E => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[2],
                    _ => JLSTZ_WALL_KICK_TESTS[2],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[1],
                    _ => JLSTZ_WALL_KICK_TESTS[1],
                },
            },
            Direction::S => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[4],
                    _ => JLSTZ_WALL_KICK_TESTS[4],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[3],
                    _ => JLSTZ_WALL_KICK_TESTS[3],
                },
            },
            Direction::W => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[6],
                    _ => JLSTZ_WALL_KICK_TESTS[6],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[5],
                    _ => JLSTZ_WALL_KICK_TESTS[5],
                },
            },
        }
    }

    fn rotate(&mut self, rotation: &Rotation) {
        self.direction = self.direction.rotate(rotation)
    }
}

#[inline(always)]
fn neg_trans(block_trans: [IVec2; 4]) -> [IVec2; 4] {
    [
        -block_trans[0],
        -block_trans[1],
        -block_trans[2],
        -block_trans[3],
    ]
}

pub struct RustominoBag {
    bag: Vec<RustominoType>, // contains the next rustomino types, shuffled
    rng: rand_xoshiro::Xoshiro256PlusPlus,
}

impl RustominoBag {
    pub fn new() -> Self {
        RustominoBag {
            bag: Vec::new(),
            rng: rand_xoshiro::Xoshiro256PlusPlus::from_entropy(),
        }
    }

    pub fn get_next(&mut self) -> Rustomino {
        // make sure the bag isn't empty
        self.fill_bag();

        let rtype = self.bag.pop().expect("rustomino bag is empty");
        log::info!("next rustomino type: {:?}", rtype);

        Rustomino::new(rtype)
    }

    // add one of each rustomino type to bag
    // then shuffle the bag
    fn fill_bag(&mut self) {
        if !self.bag.is_empty() {
            return;
        }
        self.bag.append(&mut RustominoType::iter().collect());
        self.bag.shuffle(&mut self.rng);
        log::debug!("filled rustomino bag: {:?}", self.bag);
    }
}
