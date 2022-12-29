use crate::rustomino::{RotationDirection, Rustomino, RustominoState, RustominoType};
use anyhow::{bail, Result};
use piston_window::types::Vec2d;
use std::fmt::Display;
use std::mem::discriminant;

// playfield is 10 rows wide, 20 columns high
// new rustomino's are spawned above playfield in lines 21,22
pub const PLAYFIELD_SIZE: [usize; 2] = [10, 22];
const GRAVITY_TRANSLATION: Vec2d<i32> = [0, -1];
const LEFT_TRANSLATION: Vec2d<i32> = [-1, 0];
const RIGHT_TRANSLATION: Vec2d<i32> = [1, 0];

pub enum TranslationDirection {
    Left,
    Right,
    Down,
}

impl TranslationDirection {
    pub fn get_translation(&self) -> Vec2d<i32> {
        match self {
            TranslationDirection::Left => LEFT_TRANSLATION,
            TranslationDirection::Right => RIGHT_TRANSLATION,
            TranslationDirection::Down => GRAVITY_TRANSLATION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Occupied,
    Locked(RustominoType),
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SlotState::Empty => write!(f, "  ")?,
            SlotState::Occupied => write!(f, " #")?,
            SlotState::Locked(_) => write!(f, " @")?,
        }
        Ok(())
    }
}

// RustrisBoard
#[derive(Debug)]
pub struct RustrisBoard {
    pub(crate) slots: [[SlotState; PLAYFIELD_SIZE[0] as usize]; PLAYFIELD_SIZE[1] as usize],
    pub(crate) current_rustomino: Option<Rustomino>,
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
            current_rustomino: None,
        }
    }

    // Add the rustomino to the board
    // returns false if there was a collision while adding the block (game over)
    pub fn add_new_rustomino(&mut self, rustomino: Rustomino) -> bool {
        let ok = !self.check_collision(rustomino.block_slots());
        // slots[y][x]
        self.set_slot_state(rustomino.block_slots(), SlotState::Occupied);
        self.current_rustomino = Some(rustomino);
        ok
    }

    pub fn ready_for_next(&self) -> bool {
        if let Some(current_rustomino) = &self.current_rustomino {
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
        let Some(rustomino) = &self.current_rustomino else {
            // no blocks to move/or lock
            return false;
        };

        // can't move locked blocks
        if rustomino.state == RustominoState::Locked {
            return false;
        }

        // check if moving would cause a collision
        if self.check_collision(rustomino.translated(GRAVITY_TRANSLATION)) {
            return false;
        }

        true
    }

    fn set_slot_state(&mut self, block_slots: [Vec2d<i32>; 4], new_state: SlotState) {
        log::debug!(
            "set_slot_state called block_slots: {:?} to state: {:?}",
            block_slots,
            new_state
        );
        for slot in block_slots {
            self.slots[slot[1] as usize][slot[0] as usize] = new_state;
        }
    }

    fn set_current_rustomino_slot_state(&mut self, new_state: SlotState) {
        // safely unwrap current rustomino
        if let Some(current_rustomino) = &self.current_rustomino {
            self.set_slot_state(current_rustomino.block_slots(), new_state);
        } else {
            return;
        }
    }

    fn apply_gravity(&mut self) {
        self.set_current_rustomino_slot_state(SlotState::Empty);
        // apply the gravity translation rustomino
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            log::debug!(
                "applying gravity: {:?} to {:?}",
                current_rustomino,
                current_rustomino.translated(GRAVITY_TRANSLATION),
            );
            current_rustomino.translate(GRAVITY_TRANSLATION);
        }
        self.set_current_rustomino_slot_state(SlotState::Occupied);
    }

    pub fn lock_current_rustomino(&mut self) {
        // get the current rustomino
        let current_rustomino = self.current_rustomino.as_mut().unwrap();

        // if the block can't be moved it's time to lock it
        log::debug!("locking rustomino: {:?}", current_rustomino);
        current_rustomino.lock();
        // set the block slots as locked
        for slot in current_rustomino.block_slots() {
            // slots[y][x]
            self.slots[slot[1] as usize][slot[0] as usize] =
                SlotState::Locked(current_rustomino.rustomino_type);
        }
    }

    pub fn gravity_tick(&mut self) {
        // check to see if we can move this block down

        if let Some(rustomino) = &self.current_rustomino {
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
            self.lock_current_rustomino();
        }
    }

    /// Returns the number locked rustominos on this [`RustrisBoard`].
    pub fn num_locked_rustominos(&self) -> usize {
        self.current_rustomino
            .iter()
            .filter(|x| x.state == RustominoState::Locked)
            .count()
    }

    /// Returns the get complete lines of this [`RustrisBoard`].
    pub fn get_complete_lines(&self) -> Vec<usize> {
        let mut complete_lines = vec![];
        'outer: for (i, line) in self.slots.iter().enumerate() {
            for slot in line {
                // compare variant ignoring the value
                if discriminant(slot) != discriminant(&SlotState::Locked(RustominoType::I)) {
                    continue 'outer;
                }
            }
            complete_lines.push(i);
        }
        complete_lines
    }

    pub fn clear_completed_lines(&mut self) -> Vec<usize> {
        let completed_lines = self.get_complete_lines();
        if completed_lines.len() > 0 {}
        completed_lines
    }

    /// check to see if the provided block locations collide with other locked blocks
    /// or with walls
    pub fn check_collision(&self, block_locations: [Vec2d<i32>; 4]) -> bool {
        log::debug!("check collision called: {:?}", block_locations);
        for location in block_locations {
            // check for left and right wall collisions
            if location[0] < 0 || location[0] >= PLAYFIELD_SIZE[0] as i32 {
                log::debug!("collided with left/right wall: {:?}", block_locations);
                return true;
            }
            // check for bottom wall collision
            if location[1] < 0 {
                log::debug!("collided with bottom wall: {:?}", block_locations);
                return true;
            }
            // slots[y][x] compare variant ignoring value
            if discriminant(&self.slots[location[1] as usize][location[0] as usize])
                == discriminant(&SlotState::Locked(RustominoType::I))
            {
                log::debug!("collided with locked block: {:?}", block_locations);
                return true;
            }
        }
        false
    }

    fn drop_translation(&self) -> Result<Vec2d<i32>> {
        if let Some(current_rustomino) = &self.current_rustomino {
            if current_rustomino.state == RustominoState::Locked {
                bail!("cannot move locked block");
            }

            let mut translation = GRAVITY_TRANSLATION;

            // if we can't move it down without colliding the delta is 0
            if self.check_collision(current_rustomino.translated(translation)) {
                return Ok([0, 0]);
            }

            // keep attempting to move the rustomino down until it collides and return
            // the last non-coliding translation
            loop {
                let good_translation = translation;
                translation = vecmath::vec2_add(translation, [0, -1]);
                if self.check_collision(current_rustomino.translated(translation)) {
                    return Ok(good_translation);
                }
            }
        }
        bail!("no block to move");
    }

    pub fn drop(&mut self) {
        match self.drop_translation() {
            Ok(delta) => {
                self.set_current_rustomino_slot_state(SlotState::Empty);
                self.current_rustomino.as_mut().unwrap().translate(delta);
                self.lock_current_rustomino();
            }
            Err(_) => todo!(),
        }
    }

    /// Attempt to rotate the current rustomino.
    /// Returns the rustomino rotated if it's possible
    /// Returns the unmodified rustomino if not
    pub fn rotate_current(&mut self, direction: RotationDirection) {
        // see if we can get a reference to the current rustomino
        if let Some(current_rustomino) = &self.current_rustomino {
            let rotated_blocks = current_rustomino.rotated(&direction);

            // check to see if the translation would cause a collision with a locked block
            if self.check_collision(rotated_blocks) {
                log::debug!("rotation collision detected: {:?}", rotated_blocks);
                return;
            }
        } else {
            return; // return if we can't
        }

        self.set_current_rustomino_slot_state(SlotState::Empty);

        // get mutable reference
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // perform the tranlsation
            current_rustomino.rotate(&direction);
        }

        self.set_current_rustomino_slot_state(SlotState::Occupied);
    }

    /// Attempt to translate the current rustomino.
    /// Return true if possible
    pub fn translate_current(&mut self, direction: TranslationDirection) -> bool {
        if let Some(current_rustomino) = &self.current_rustomino {
            let translated_blocks = current_rustomino.translated(direction.get_translation());

            // check to see if the translation would cause a collision with a locked block
            if self.check_collision(translated_blocks) {
                return false;
            }
        } else {
            return false;
        }

        self.set_current_rustomino_slot_state(SlotState::Empty);

        // get mutable reference
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // perform the tranlsation
            current_rustomino.translate(direction.get_translation());
        } else {
            return false;
        }

        self.set_current_rustomino_slot_state(SlotState::Occupied);
        true
    }
}
