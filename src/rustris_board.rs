use crate::rustominos::{Rustomino, RustominoState};
use anyhow::{bail, Result};
use piston_window::{types::Color, types::Vec2d, Context, G2d};
use std::{fmt::Display, thread::current};

// playfield is 10 rows wide, 20 columns high
// new rustomino's are spawned above playfield in lines 21,22
const PLAYFIELD_SIZE: [usize; 2] = [10, 22];
const GRAVITY_TRANSLATION: Vec2d<i32> = [0, -1];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotState {
    Empty,
    Occupied,
    Locked,
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SlotState::Empty => write!(f, "  ")?,
            SlotState::Occupied => write!(f, " #")?,
            SlotState::Locked => write!(f, " @")?,
        }
        Ok(())
    }
}

// RustrisBoard
#[derive(Debug)]
pub struct RustrisBoard {
    slots: [[SlotState; PLAYFIELD_SIZE[0]]; PLAYFIELD_SIZE[1]],
    rustominos: Vec<Rustomino>,
}

// impl Display for Rustomino {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, " ")?;
//         write!(f, "{}", "-".repeat((self.bounding_box[0] * 2 - 1) as usize))?;
//         write!(f, " \n")?;
//         for line in (0..self.bounding_box[1]).rev() {
//             write!(f, "|")?;
//             'row: for row in 0..self.bounding_box[0] {
//                 for block in self.blocks {
//                     if block[0] == row && block[1] == line {
//                         if row != (self.bounding_box[1] - 1) {
//                             write!(f, "# ")?;
//                         } else {
//                             write!(f, "#")?;
//                         }
//                         continue 'row;
//                     }
//                 }
//                 if row != (self.bounding_box[1] - 1) {
//                     write!(f, "  ")?;
//                 } else {
//                     write!(f, " ")?;
//                 }
//             }
//             write!(f, "|\n")?;
//         }
//         write!(f, " ")?;
//         write!(f, "{}", "-".repeat((self.bounding_box[0] * 2 - 1) as usize))?;
//         write!(f, " ")?;
//         Ok(())
//     }
// }

impl Display for RustrisBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "-".repeat(PLAYFIELD_SIZE[0] * 2))?;
        for row in self.slots.iter().rev() {
            for slot in row {
                write!(f, "{}", slot)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "{}", "-".repeat(PLAYFIELD_SIZE[0] * 2))?;
        Ok(())
    }
}

impl RustrisBoard {
    pub fn new() -> Self {
        log::info!("Initializing Rustris Board");
        RustrisBoard {
            slots: [[SlotState::Empty; PLAYFIELD_SIZE[0]]; PLAYFIELD_SIZE[1]],
            rustominos: Vec::new(),
        }
    }

    fn set_slot_state(&mut self, rustomino: &Rustomino, state: SlotState) {
        log::debug!(
            "set_slot_state called rustomino: {:?} block_slots: {:?} to state: {:?}",
            rustomino,
            rustomino.block_slots(),
            state
        );
        for slot in rustomino.block_slots() {
            self.slots[slot[1] as usize][slot[0] as usize] = state;
        }
    }

    pub fn add_new_rustomino(&mut self, rustomino: Rustomino) {
        // slots[y][x]
        self.set_slot_state(&rustomino, SlotState::Occupied);
        self.rustominos.push(rustomino);
    }

    pub fn check_need_next(&self) -> bool {
        if let Some(current_rustomino) = self.rustominos.last() {
            if current_rustomino.state == RustominoState::Locked {
                return true;
            }
            false
        } else {
            true
        }
    }

    pub fn gravity_tick(&mut self) {
        // check to see if we can move this block down
        // if we can translate it

        let mut movable = true;
        {
            let Some(rustomino) = self.rustominos.last() else {
                return;
            };

            // this shouldn't happen I don't think, but it's here JIC
            if rustomino.state == RustominoState::Locked {
                return;
            }

            if self.check_collision(rustomino.translated(GRAVITY_TRANSLATION)) {
                movable = false;
            }
        }

        // we should have returned if there was no last block
        let rustomino = self.rustominos.last_mut().unwrap();

        if movable {
            log::debug!(
                "moving rustomino: {:?} to {:?}",
                rustomino,
                rustomino.translated(GRAVITY_TRANSLATION),
            );
            // set the previous slots as empty
            for slot in rustomino.block_slots() {
                // slots[y][x]
                self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Empty;
            }
            rustomino.translate(GRAVITY_TRANSLATION);
            // set the new slots as occupied
            for slot in rustomino.block_slots() {
                // slots[y][x]
                self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Occupied;
            }
        } else {
            // if the block can't be moved it's time to lock it
            log::debug!("locking rustomino: {:?}", rustomino);
            rustomino.lock();
            // set the block slots as locked
            for slot in rustomino.block_slots() {
                // slots[y][x]
                self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Locked;
            }
        }
    }

    /// Returns the number locked rustominos on this [`RustrisBoard`].
    pub fn num_locked_rustominos(&self) -> usize {
        self.rustominos
            .iter()
            .filter(|x| x.state == RustominoState::Locked)
            .count()
    }

    /// Returns the get complete lines of this [`RustrisBoard`].
    pub fn get_complete_lines(&self) -> Vec<usize> {
        let mut complete_lines = vec![];
        'outer: for (i, line) in self.slots.iter().enumerate() {
            for slot in line {
                if *slot != SlotState::Locked {
                    continue 'outer;
                }
            }
            complete_lines.push(i);
        }
        complete_lines
    }

    /// check to see if the provided block locations collide with other locked blocks
    pub fn check_collision(&self, block_locations: [Vec2d<i32>; 4]) -> bool {
        for location in block_locations {
            // slots[y][x]
            if self.slots[location[1] as usize][location[0] as usize] == SlotState::Locked {
                return true;
            }
        }
        false
    }

    pub fn get_lowest_possible_slots(&self) -> Result<[Vec2d<i32>; 4]> {
        if let Some(current_rustomino) = self.rustominos.last() {
            if current_rustomino.state == RustominoState::Locked {
                bail!("cannot move locked block");
            }

            let mut translation = GRAVITY_TRANSLATION;

            if self.check_collision(current_rustomino.translated(translation)) {
                return Ok(current_rustomino.block_slots());
            }

            // keep attempting to move the rustomino down until it collides and return
            // the last non-coliding translation
            loop {
                let good_translation = translation;
                translation = vecmath::vec2_add(translation, [0, -1]);
                if self.check_collision(current_rustomino.translated(translation)) {
                    return Ok(current_rustomino.translated(good_translation));
                }
            }
        } else {
            bail!("no block to move");
        }
    }

    /// Attempt to rotate the provided rustomino.
    /// Returns the rustomino rotated if it's possible
    /// Returns the unmodified rustomino if not
    pub fn rotate_rustomino<'a>(&self, rustomino: &'a mut Rustomino) -> &'a Rustomino {
        todo!()
    }
}
