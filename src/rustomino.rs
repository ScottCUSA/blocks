use macroquad::{
    prelude::*,
};

use std::fmt::Display;
use strum::EnumIter;

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
    const CYAN: Color = Color::new(0.0, 0.9, 1.0, 1.0); 
    const YELLOW: Color = Color::new(1.0, 0.87, 0.0, 1.0);
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
    pub rustomino_type: RustominoType,
    pub rotation: RustominoRotation,
    pub blocks: [IVec2; 4],
    pub translation: IVec2,
    bounding_box: IVec2,
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

    pub fn reset(self) -> Rustomino {
        Rustomino::new(self.rustomino_type)
    }

    pub fn translate(&mut self, delta: IVec2) {
        log::debug!("translate called: delta {:?}", delta);
        self.translation = self.translation + delta;
    }

    pub fn translated(&self, delta: IVec2) -> [IVec2; 4] {
        [
            (self.blocks[0] + self.translation) + delta,
            (self.blocks[1] + self.translation) + delta,
            (self.blocks[2] + self.translation) + delta,
            (self.blocks[3] + self.translation) + delta,
        ]
    }

    pub fn board_slots(&self) -> [IVec2; 4] {
        self.translated(IVec2::ZERO)
    }

    /// .
    pub fn rotate(&mut self, direction: &RotationDirection) {
        let translation = self.rotation.get_translation(direction);

        self.blocks = [
            self.blocks[0] + translation.0[0],
            self.blocks[1] + translation.0[1],
            self.blocks[2] + translation.0[2],
            self.blocks[3] + translation.0[3],
        ];

        self.rotation.rotate(direction);
    }

    pub fn rotated(&self, direction: &RotationDirection) -> [IVec2; 4] {
        let rotation = self.rotation.get_translation(direction);

        [
            (self.blocks[0] + self.translation) + rotation.0[0],
            (self.blocks[1] + self.translation) + rotation.0[1],
            (self.blocks[2] + self.translation) + rotation.0[2],
            (self.blocks[3] + self.translation) + rotation.0[3],
        ]
    }
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
