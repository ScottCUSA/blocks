use macroquad::prelude::*;

use ::rand::{seq::SliceRandom, SeedableRng};
use std::fmt::Display;
use strum::{EnumIter, IntoEnumIterator};

const I_BOUNDING_BOX: IVec2 = ivec2(4, 4);
const O_BOUNDING_BOX: IVec2 = ivec2(4, 3);
const T_L_J_S_Z_BOUNDING_BOX: IVec2 = ivec2(3, 3);

const I_START_TRANSLATION: IVec2 = ivec2(3, 18);
const O_T_L_J_S_Z_START_TRANSLATION: IVec2 = ivec2(3, 19);

const I_BLOCKS: [IVec2; 4] = [ivec2(0, 2), ivec2(1, 2), ivec2(2, 2), ivec2(3, 2)];
const O_BLOCKS: [IVec2; 4] = [ivec2(1, 2), ivec2(2, 2), ivec2(2, 1), ivec2(1, 1)];
const T_BLOCKS: [IVec2; 4] = [ivec2(1, 1), ivec2(0, 1), ivec2(1, 2), ivec2(2, 1)];
const L_BLOCKS: [IVec2; 4] = [ivec2(1, 1), ivec2(0, 1), ivec2(2, 2), ivec2(2, 1)];
const J_BLOCKS: [IVec2; 4] = [ivec2(1, 1), ivec2(0, 1), ivec2(0, 2), ivec2(2, 1)];
const S_BLOCKS: [IVec2; 4] = [ivec2(1, 1), ivec2(0, 1), ivec2(1, 2), ivec2(2, 2)];
const Z_BLOCKS: [IVec2; 4] = [ivec2(1, 1), ivec2(0, 2), ivec2(1, 2), ivec2(2, 1)];

const I_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(2, 1),
        ivec2(1, 0),
        ivec2(0, -1),
        ivec2(-1, -2),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(1, -2),
        ivec2(0, -1),
        ivec2(-1, 0),
        ivec2(-2, 1),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(-2, -1),
        ivec2(-1, 0),
        ivec2(0, 1),
        ivec2(1, 2),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(-1, 2),
        ivec2(0, 1),
        ivec2(1, 0),
        ivec2(2, -1),
    ],
];

const O_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(1, 0),
        ivec2(0, -1),
        ivec2(-1, 0),
        ivec2(0, 1),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, -1),
        ivec2(-1, 0),
        ivec2(0, 1),
        ivec2(1, 0),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(-1, 0),
        ivec2(0, 1),
        ivec2(1, 0),
        ivec2(0, -1),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 1),
        ivec2(1, 0),
        ivec2(0, -1),
        ivec2(-1, 0),
    ],
];

const T_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(0, 0),
        ivec2(1, 1),
        ivec2(1, -1),
        ivec2(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, 0),
        ivec2(1, -1),
        ivec2(-1, -1),
        ivec2(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(0, 0),
        ivec2(-1, -1),
        ivec2(-1, 1),
        ivec2(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 0),
        ivec2(-1, 1),
        ivec2(1, 1),
        ivec2(1, -1),
    ],
];

const L_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(0, 0),
        ivec2(1, 1),
        ivec2(0, -2),
        ivec2(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, 0),
        ivec2(1, -1),
        ivec2(-2, 0),
        ivec2(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(0, 0),
        ivec2(-1, -1),
        ivec2(0, 2),
        ivec2(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 0),
        ivec2(-1, 1),
        ivec2(2, 0),
        ivec2(1, -1),
    ],
];

const J_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(0, 0),
        ivec2(1, 1),
        ivec2(2, 0),
        ivec2(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, 0),
        ivec2(1, -1),
        ivec2(0, -2),
        ivec2(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(0, 0),
        ivec2(-1, -1),
        ivec2(-2, 0),
        ivec2(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 0),
        ivec2(-1, 1),
        ivec2(0, 2),
        ivec2(1, -1),
    ],
];

const S_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(0, 0),
        ivec2(1, 1),
        ivec2(1, -1),
        ivec2(0, -2),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, 0),
        ivec2(1, -1),
        ivec2(-1, -1),
        ivec2(-2, 0),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(0, 0),
        ivec2(-1, -1),
        ivec2(-1, 1),
        ivec2(0, 2),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 0),
        ivec2(-1, 1),
        ivec2(1, 1),
        ivec2(2, 0),
    ],
];

const Z_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        ivec2(0, 0),
        ivec2(2, 0),
        ivec2(1, -1),
        ivec2(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        ivec2(0, 0),
        ivec2(0, -2),
        ivec2(-1, -1),
        ivec2(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        ivec2(0, 0),
        ivec2(-2, 0),
        ivec2(-1, 1),
        ivec2(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        ivec2(0, 0),
        ivec2(0, 2),
        ivec2(1, 1),
        ivec2(1, -1),
    ],
];

const JLSTZ_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E (0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(-1, 1),
        ivec2(0, -2),
        ivec2(-1, -2),
    ],
    [
        // E->N (0, 0),(1, 0),(1,-1),( 0,2),(1,2)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(1, -1),
        ivec2(0, 2),
        ivec2(1, 2),
    ],
    [
        // E->S (0, 0),(1, 0),(1,-1),(0, 2),(1, 2)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(1, -1),
        ivec2(0, 2),
        ivec2(1, 2),
    ],
    [
        // S->E ( 0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(-1, 1),
        ivec2(0, -2),
        ivec2(-1, -2),
    ],
    [
        // E->W ( 0, 0),(1, 0),(1,1),( 0,-2),(1,-2)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(1, 1),
        ivec2(0, -2),
        ivec2(1, -2),
    ],
    [
        // W->E ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(-1, -1),
        ivec2(0, 2),
        ivec2(-1, 2),
    ],
    [
        // W->N ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(-1, -1),
        ivec2(0, 2),
        ivec2(-1, 2),
    ],
    [
        // N->W (0, 0),(1, 0),(1, 1),(0, -2),(1, -2)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(1, 1),
        ivec2(0, -2),
        ivec2(1, -2),
    ],
];

const I_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        ivec2(0, 0),
        ivec2(-2, 0),
        ivec2(1, 0),
        ivec2(-2, -1),
        ivec2(1, 2),
    ],
    [
        // E->N ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        ivec2(0, 0),
        ivec2(2, 0),
        ivec2(-1, 0),
        ivec2(2, 1),
        ivec2(-1, -2),
    ],
    [
        // E->S ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(2, 0),
        ivec2(-1, 2),
        ivec2(2, -1),
    ],
    [
        // S->E ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(-2, 0),
        ivec2(1, -2),
        ivec2(-2, 1),
    ],
    [
        // E->W ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        ivec2(0, 0),
        ivec2(2, 0),
        ivec2(-1, 0),
        ivec2(2, 1),
        ivec2(-1, -2),
    ],
    [
        // W->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        ivec2(0, 0),
        ivec2(-2, 0),
        ivec2(1, 0),
        ivec2(-2, -1),
        ivec2(1, 2),
    ],
    [
        // W->N ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        ivec2(0, 0),
        ivec2(1, 0),
        ivec2(-2, 0),
        ivec2(1, -2),
        ivec2(-2, 1),
    ],
    [
        // N->W ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        ivec2(0, 0),
        ivec2(-1, 0),
        ivec2(2, 0),
        ivec2(-1, 2),
        ivec2(2, -1),
    ],
];

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
    const BLUE: Color = Color::new(0.09, 0.0, 1.0, 1.0);
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

#[derive(Debug, Clone)]
pub struct Rustomino {
    pub rtype: RustominoType,
    pub state: RustominoState,
    pub rotation: RustominoRotation,
    pub blocks: [IVec2; 4],
    pub translation: IVec2,
    bounding_box: IVec2,
}

impl Rustomino {
    pub fn new(block_type: RustominoType) -> Rustomino {
        match block_type {
            RustominoType::I => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(I_ROTATIONS),
                blocks: I_BLOCKS,
                translation: I_START_TRANSLATION,
                bounding_box: I_BOUNDING_BOX,
            },
            RustominoType::O => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(O_ROTATIONS),
                blocks: O_BLOCKS,
                bounding_box: O_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::T => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(T_ROTATIONS),
                blocks: T_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::L => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(L_ROTATIONS),
                blocks: L_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::J => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(J_ROTATIONS),
                blocks: J_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::S => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(S_ROTATIONS),
                blocks: S_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::Z => Rustomino {
                rtype: block_type,
                state: RustominoState::Falling { time: 0. },
                rotation: RustominoRotation::new(Z_ROTATIONS),
                blocks: Z_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
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

    pub fn rotate(&mut self, direction: &RotationDirection, translation: &IVec2) {
        let rotation_trans = self.rotation.get_translation(direction);

        self.blocks = [
            self.blocks[0] + rotation_trans.0[0] + *translation,
            self.blocks[1] + rotation_trans.0[1] + *translation,
            self.blocks[2] + rotation_trans.0[2] + *translation,
            self.blocks[3] + rotation_trans.0[3] + *translation,
        ];

        self.rotation.rotate(direction);
    }

    pub fn rotated(&self, direction: &RotationDirection) -> [IVec2; 4] {
        let rotation = self.rotation.get_translation(direction);

        [
            self.blocks[0] + self.translation + rotation.0[0],
            self.blocks[1] + self.translation + rotation.0[1],
            self.blocks[2] + self.translation + rotation.0[2],
            self.blocks[3] + self.translation + rotation.0[3],
        ]
    }

    pub fn wall_kick_tests(&self, direction: &RotationDirection) -> [IVec2; 5] {
        self.rotation.get_wall_kick_tests(self.rtype, direction)
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

impl Display for Rustomino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        write!(f, "{}", "-".repeat((self.bounding_box[0] * 2 - 1) as usize))?;
        writeln!(f, " ")?;
        for line in (0..self.bounding_box[1]).rev() {
            write!(f, "|")?;
            'row: for row in 0..self.bounding_box[0] {
                for block in self.blocks {
                    if block[0] == row && block[1] == line {
                        if row != (self.bounding_box[1] - 1) {
                            write!(f, "# ")?;
                        } else {
                            write!(f, "#")?;
                        }
                        continue 'row;
                    }
                }
                if row != (self.bounding_box[1] - 1) {
                    write!(f, "  ")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "|")?;
        }
        write!(f, " ")?;
        write!(f, "{}", "-".repeat((self.bounding_box[0] * 2 - 1) as usize))?;
        write!(f, " ")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RustominoDirection {
    N,
    E,
    S,
    W,
}

impl RustominoDirection {
    fn rotate(&self, direction: &RotationDirection) -> RustominoDirection {
        match self {
            RustominoDirection::N => match direction {
                RotationDirection::Cw => RustominoDirection::E,
                RotationDirection::Ccw => RustominoDirection::W,
            },
            RustominoDirection::E => match direction {
                RotationDirection::Cw => RustominoDirection::S,
                RotationDirection::Ccw => RustominoDirection::N,
            },
            RustominoDirection::S => match direction {
                RotationDirection::Cw => RustominoDirection::W,
                RotationDirection::Ccw => RustominoDirection::E,
            },
            RustominoDirection::W => match direction {
                RotationDirection::Cw => RustominoDirection::N,
                RotationDirection::Ccw => RustominoDirection::S,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum RotationDirection {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy)]
pub struct RustominoRotation {
    direction: RustominoDirection,
    n2e: RotationTranslation,
    e2s: RotationTranslation,
    s2w: RotationTranslation,
    w2n: RotationTranslation,
}

impl RustominoRotation {
    fn new(values: [[IVec2; 4]; 4]) -> Self {
        Self {
            direction: RustominoDirection::N,
            n2e: RotationTranslation::new(values[0]),
            e2s: RotationTranslation::new(values[1]),
            s2w: RotationTranslation::new(values[2]),
            w2n: RotationTranslation::new(values[3]),
        }
    }

    fn get_translation(&self, direction: &RotationDirection) -> RotationTranslation {
        match self.direction {
            RustominoDirection::N => match direction {
                RotationDirection::Cw => self.n2e,
                RotationDirection::Ccw => -self.w2n,
            },
            RustominoDirection::E => match direction {
                RotationDirection::Cw => self.e2s,
                RotationDirection::Ccw => -self.n2e,
            },
            RustominoDirection::S => match direction {
                RotationDirection::Cw => self.s2w,
                RotationDirection::Ccw => -self.e2s,
            },
            RustominoDirection::W => match direction {
                RotationDirection::Cw => self.w2n,
                RotationDirection::Ccw => -self.s2w,
            },
        }
    }

    fn get_wall_kick_tests(
        &self,
        rtype: RustominoType,
        direction: &RotationDirection,
    ) -> [IVec2; 5] {
        match self.direction {
            RustominoDirection::N => match direction {
                RotationDirection::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[0],
                    _ => JLSTZ_WALL_KICK_TESTS[0],
                },
                RotationDirection::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[7],
                    _ => JLSTZ_WALL_KICK_TESTS[7],
                },
            },
            RustominoDirection::E => match direction {
                RotationDirection::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[2],
                    _ => JLSTZ_WALL_KICK_TESTS[2],
                },
                RotationDirection::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[1],
                    _ => JLSTZ_WALL_KICK_TESTS[1],
                },
            },
            RustominoDirection::S => match direction {
                RotationDirection::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[4],
                    _ => JLSTZ_WALL_KICK_TESTS[4],
                },
                RotationDirection::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[3],
                    _ => JLSTZ_WALL_KICK_TESTS[3],
                },
            },
            RustominoDirection::W => match direction {
                RotationDirection::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[6],
                    _ => JLSTZ_WALL_KICK_TESTS[6],
                },
                RotationDirection::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[5],
                    _ => JLSTZ_WALL_KICK_TESTS[5],
                },
            },
        }
    }

    fn rotate(&mut self, direction: &RotationDirection) {
        self.direction = self.direction.rotate(direction)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RotationTranslation([IVec2; 4]);

impl RotationTranslation {
    fn new(values: [IVec2; 4]) -> Self {
        RotationTranslation(values)
    }
}

impl std::ops::Neg for RotationTranslation {
    type Output = Self;

    fn neg(self) -> Self {
        Self([-self.0[0], -self.0[1], -self.0[2], -self.0[3]])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RustominoState {
    Falling { time: f64 },
    Lockdown { time: f64 },
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

    pub fn get_next_rustomino(&mut self) -> Rustomino {
        // make sure the bag isn't empty
        self.fill_rustomino_bag();

        let rtype = self.bag.pop().unwrap();
        log::info!("next rustomino type: {:?}", rtype);

        Rustomino::new(rtype)
    }

    // add one of each rustomino type to bag
    // then shuffle the bag
    fn fill_rustomino_bag(&mut self) {
        if !self.bag.is_empty() {
            log::trace!("rustomino bag is not empty: {:?}", self.bag);
            return;
        }
        self.bag.append(&mut RustominoType::iter().collect());
        self.bag.shuffle(&mut self.rng);
        log::debug!("filled rustomino bag: {:?}", self.bag);
    }
}
