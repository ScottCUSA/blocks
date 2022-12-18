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

    fn can_fall(&self) -> bool {
        // get the current rustomino
        let Some(rustomino) = self.rustominos.last() else {
            // no blocks to move/or lock
            return false;
        };

        // can't move locked blocks
        if rustomino.state == RustominoState::Locked {
            return false;
        }

        // Check if block is at bottom of play area
        for block in rustomino.block_slots() {
            // block[x, y]
            if block[1] == 0 {
                return false;
            }
        }

        // check if moving would cause a collision
        if self.check_collision(rustomino.translated(GRAVITY_TRANSLATION)) {
            return false;
        }

        true
    }

    fn apply_gravity(&mut self) {
        // get the current rustomino
        let current_rustomino = self.rustominos.last_mut().unwrap();

        log::debug!(
            "moving rustomino: {:?} to {:?}",
            current_rustomino,
            current_rustomino.translated(GRAVITY_TRANSLATION),
        );
        // set the previous slots as empty
        for slot in current_rustomino.block_slots() {
            // slots[y][x]
            self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Empty;
        }
        current_rustomino.translate(GRAVITY_TRANSLATION);
        // set the new slots as occupied
        for slot in current_rustomino.block_slots() {
            // slots[y][x]
            self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Occupied;
        }
    }

    fn lock_current_block(&mut self) {
        // get the current rustomino
        let current_rustomino = self.rustominos.last_mut().unwrap();

        // if the block can't be moved it's time to lock it
        log::debug!("locking rustomino: {:?}", current_rustomino);
        current_rustomino.lock();
        // set the block slots as locked
        for slot in current_rustomino.block_slots() {
            // slots[y][x]
            self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Locked;
        }
    }

    pub fn gravity_tick(&mut self) {
        // check to see if we can move this block down

        if let Some(rustomino) = self.rustominos.last() {
            // ignore locked blocks
            if rustomino.state == RustominoState::Locked {
                return;
            }
        };

        // check to see if the current rustomino can fall
        let movable = self.can_fall();

        if movable {
            self.apply_gravity();
        } else {
            self.lock_current_block();
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

    pub fn lowest_possible_position(&self) -> Result<[Vec2d<i32>; 4]> {
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
