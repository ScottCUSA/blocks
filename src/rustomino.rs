use piston_window::types::Vec2d;
use rand::distributions::{Distribution, Standard};
use std::fmt::Display;
use strum::EnumIter;

const I_BOUNDING_BOX: Vec2d<i32> = [4, 4];
const O_BOUNDING_BOX: Vec2d<i32> = [4, 3];
const T_L_J_S_Z_BOUNDING_BOX: Vec2d<i32> = [3, 3];

const I_START_TRANSLATION: Vec2d<i32> = [3, 18];
const O_T_L_J_S_Z_START_TRANSLATION: Vec2d<i32> = [3, 19];

const I_BLOCKS: [Vec2d<i32>; 4] = [[0, 2], [1, 2], [2, 2], [3, 2]];
const O_BLOCKS: [Vec2d<i32>; 4] = [[1, 2], [2, 2], [2, 1], [1, 1]];
const T_BLOCKS: [Vec2d<i32>; 4] = [[1, 1], [0, 1], [1, 2], [2, 1]];
const L_BLOCKS: [Vec2d<i32>; 4] = [[1, 1], [0, 1], [2, 2], [2, 1]];
const J_BLOCKS: [Vec2d<i32>; 4] = [[1, 1], [0, 1], [0, 2], [2, 1]];
const S_BLOCKS: [Vec2d<i32>; 4] = [[1, 1], [0, 1], [1, 2], [2, 2]];
const Z_BLOCKS: [Vec2d<i32>; 4] = [[1, 1], [0, 2], [1, 2], [2, 1]];

const I_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [2, 1],
        [1, 0],
        [0, -1],
        [-1, -2],
    ],
    [
        // E>>S || -(S>>E)
        [1, -2],
        [0, -1],
        [-1, 0],
        [-2, 1],
    ],
    [
        // S>>W || -(W>>S)
        [-2, -1],
        [-1, 0],
        [0, 1],
        [1, 2],
    ],
    [
        // W>>N || -(N>>W)
        [-1, 2],
        [0, 1],
        [1, 0],
        [2, -1],
    ],
];

const O_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [1, 0],
        [0, -1],
        [-1, 0],
        [0, 1],
    ],
    [
        // E>>S || -(S>>E)
        [0, -1],
        [-1, 0],
        [0, 1],
        [1, 0],
    ],
    [
        // S>>W || -(W>>S)
        [-1, 0],
        [0, 1],
        [1, 0],
        [0, -1],
    ],
    [
        // W>>N || -(N>>W)
        [0, 1],
        [1, 0],
        [0, -1],
        [-1, 0],
    ],
];

const T_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [0, 0],
        [1, 1],
        [1, -1],
        [-1, -1],
    ],
    [
        // E>>S || -(S>>E)
        [0, 0],
        [1, -1],
        [-1, -1],
        [-1, 1],
    ],
    [
        // S>>W || -(W>>S)
        [0, 0],
        [-1, -1],
        [-1, 1],
        [1, 1],
    ],
    [
        // W>>N || -(N>>W)
        [0, 0],
        [-1, 1],
        [1, 1],
        [1, -1],
    ],
];

const L_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [0, 0],
        [1, 1],
        [0, -2],
        [-1, -1],
    ],
    [
        // E>>S || -(S>>E)
        [0, 0],
        [1, -1],
        [-2, 0],
        [-1, 1],
    ],
    [
        // S>>W || -(W>>S)
        [0, 0],
        [-1, -1],
        [0, 2],
        [1, 1],
    ],
    [
        // W>>N || -(N>>W)
        [0, 0],
        [-1, 1],
        [2, 0],
        [1, -1],
    ],
];

const J_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [0, 0],
        [1, 1],
        [2, 0],
        [-1, -1],
    ],
    [
        // E>>S || -(S>>E)
        [0, 0],
        [1, -1],
        [0, -2],
        [-1, 1],
    ],
    [
        // S>>W || -(W>>S)
        [0, 0],
        [-1, -1],
        [-2, 0],
        [1, 1],
    ],
    [
        // W>>N || -(N>>W)
        [0, 0],
        [-1, 1],
        [0, 2],
        [1, -1],
    ],
];

const S_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [0, 0],
        [1, 1],
        [1, -1],
        [0, -2],
    ],
    [
        // E>>S || -(S>>E)
        [0, 0],
        [1, -1],
        [-1, -1],
        [-2, 0],
    ],
    [
        // S>>W || -(W>>S)
        [0, 0],
        [-1, -1],
        [-1, 1],
        [0, 2],
    ],
    [
        // W>>N || -(N>>W)
        [0, 0],
        [-1, 1],
        [1, 1],
        [2, 0],
    ],
];

const Z_ROTATIONS: [[Vec2d<i32>; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        [0, 0],
        [2, 0],
        [1, -1],
        [-1, -1],
    ],
    [
        // E>>S || -(S>>E)
        [0, 0],
        [0, -2],
        [-1, -1],
        [-1, 1],
    ],
    [
        // S>>W || -(W>>S)
        [0, 0],
        [-2, 0],
        [-1, 1],
        [1, 1],
    ],
    [
        // W>>N || -(N>>W)
        [0, 0],
        [0, 2],
        [1, 1],
        [1, -1],
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

// use rand::distributions::{Distribution, Standard};
// use rand::SeedableRng;
// let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(123);
// let values: Vec<RustominoType> = Standard.sample_iter(&mut rng).take(50).collect();

// println!("{:?}", values);

/// Allow random generation for RustominoTypes
impl Distribution<RustominoType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> RustominoType {
        match rng.gen_range(0..7) {
            0 => RustominoType::I,
            1 => RustominoType::O,
            2 => RustominoType::T,
            3 => RustominoType::L,
            4 => RustominoType::J,
            5 => RustominoType::S,
            6 => RustominoType::Z,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rustomino {
    pub rustomino_type: RustominoType,
    pub rotation: RustominoRotation,
    pub blocks: [Vec2d<i32>; 4],
    pub translation: Vec2d<i32>,
    bounding_box: Vec2d<i32>,
}

impl Rustomino {
    pub fn new(block_type: RustominoType) -> Rustomino {
        match block_type {
            RustominoType::I => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(I_ROTATIONS),
                blocks: I_BLOCKS,
                translation: I_START_TRANSLATION,
                bounding_box: I_BOUNDING_BOX,
            },
            RustominoType::O => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(O_ROTATIONS),
                blocks: O_BLOCKS,
                bounding_box: O_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::T => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(T_ROTATIONS),
                blocks: T_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::L => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(L_ROTATIONS),
                blocks: L_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::J => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(J_ROTATIONS),
                blocks: J_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::S => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(S_ROTATIONS),
                blocks: S_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
            RustominoType::Z => Rustomino {
                rustomino_type: block_type,
                rotation: RustominoRotation::new(Z_ROTATIONS),
                blocks: Z_BLOCKS,
                bounding_box: T_L_J_S_Z_BOUNDING_BOX,
                translation: O_T_L_J_S_Z_START_TRANSLATION,
            },
        }
    }

    pub fn translate(&mut self, delta: Vec2d<i32>) {
        log::debug!("translate called: delta {:?}", delta);
        self.translation = vecmath::vec2_add(self.translation, delta);
    }

    pub fn translated(&self, delta: Vec2d<i32>) -> [Vec2d<i32>; 4] {
        [
            vecmath::vec2_add(vecmath::vec2_add(self.blocks[0], self.translation), delta),
            vecmath::vec2_add(vecmath::vec2_add(self.blocks[1], self.translation), delta),
            vecmath::vec2_add(vecmath::vec2_add(self.blocks[2], self.translation), delta),
            vecmath::vec2_add(vecmath::vec2_add(self.blocks[3], self.translation), delta),
        ]
    }

    pub fn board_slots(&self) -> [Vec2d<i32>; 4] {
        self.translated([0, 0])
    }

    /// .
    pub fn rotate(&mut self, direction: &RotationDirection) {
        let translation = self.rotation.get_translation(direction);

        self.blocks = [
            vecmath::vec2_add(self.blocks[0], translation.0[0]),
            vecmath::vec2_add(self.blocks[1], translation.0[1]),
            vecmath::vec2_add(self.blocks[2], translation.0[2]),
            vecmath::vec2_add(self.blocks[3], translation.0[3]),
        ];

        self.rotation.rotate(direction);
    }

    pub fn rotated(&self, direction: &RotationDirection) -> [Vec2d<i32>; 4] {
        let rotation = self.rotation.get_translation(direction);

        [
            vecmath::vec2_add(
                vecmath::vec2_add(self.blocks[0], self.translation),
                rotation.0[0],
            ),
            vecmath::vec2_add(
                vecmath::vec2_add(self.blocks[1], self.translation),
                rotation.0[1],
            ),
            vecmath::vec2_add(
                vecmath::vec2_add(self.blocks[2], self.translation),
                rotation.0[2],
            ),
            vecmath::vec2_add(
                vecmath::vec2_add(self.blocks[3], self.translation),
                rotation.0[3],
            ),
        ]
    }
}

impl Display for Rustomino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        write!(f, "{}", "-".repeat((self.bounding_box[0] * 2 - 1) as usize))?;
        write!(f, " \n")?;
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
            write!(f, "|\n")?;
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
    fn new(values: [[Vec2d<i32>; 4]; 4]) -> Self {
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

    fn rotate(&mut self, direction: &RotationDirection) {
        self.direction = self.direction.rotate(direction)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RotationTranslation([Vec2d<i32>; 4]);

impl RotationTranslation {
    fn new(values: [Vec2d<i32>; 4]) -> Self {
        RotationTranslation(values)
    }
}

impl std::ops::Neg for RotationTranslation {
    type Output = Self;

    fn neg(self) -> Self {
        Self([
            vecmath::vec2_neg(self.0[0]),
            vecmath::vec2_neg(self.0[1]),
            vecmath::vec2_neg(self.0[2]),
            vecmath::vec2_neg(self.0[3]),
        ])
    }
}
