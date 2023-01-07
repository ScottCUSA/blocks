use std::{fmt::Display, mem::discriminant};

use macroquad::prelude::*;

use crate::rustomino::{RotationDirection, Rustomino, RustominoType};

pub(crate) const BOARD_SLOTS: [i32; 2] = [10, 22];
pub(crate) const PLAYFIELD_SIZE: [i32; 2] = [10, 20];

// RustrisBoard
#[derive(Debug)]
pub struct RustrisBoard {
    pub(crate) slots: [[SlotState; BOARD_SLOTS[0] as usize]; BOARD_SLOTS[1] as usize],
    pub(crate) current_rustomino: Option<Rustomino>,
    pub(crate) ghost_rustomino: Option<Rustomino>,
}

impl RustrisBoard {
    pub fn new() -> Self {
        log::info!("Initializing Rustris Board");
        RustrisBoard {
            slots: [[SlotState::Empty; BOARD_SLOTS[0] as usize]; BOARD_SLOTS[1] as usize],
            current_rustomino: None,
            ghost_rustomino: None,
        }
    }

    /// Adds a new rustomino to the board
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn set_current_rustomino(&mut self, rustomino: Rustomino) -> bool {
        let ok = !self.check_collision(rustomino.board_slots());
        // slots[y][x]
        self.set_slot_states(rustomino.board_slots(), SlotState::Occupied);
        self.ghost_rustomino = Some(rustomino.clone());
        self.current_rustomino = Some(rustomino);
        self.update_ghost_rustomino();
        ok
    }

    /// checks to see if the board needs the next rustomino
    pub fn ready_for_next(&self) -> bool {
        self.current_rustomino.is_none()
    }

    pub fn can_fall(&self) -> bool {
        // get the current rustomino
        let Some(rustomino) = &self.current_rustomino else {
            // no blocks to move/or lock
            return false;
        };

        // check if moving would cause a collision
        if self.check_collision(rustomino.translated(TranslationDirection::DOWN_TRANSLATION)) {
            return false;
        }

        true
    }

    fn set_slot_states(&mut self, block_slots: [IVec2; 4], new_state: SlotState) {
        log::debug!(
            "set_slot_state called block_slots: {:?} to state: {:?}",
            block_slots,
            new_state
        );
        for slot in block_slots {
            self.slots[slot[1] as usize][slot[0] as usize] = new_state;
        }
    }

    /// apply gravity to the current rustomino
    pub fn apply_gravity(&mut self) {
        self.set_rustomino_slot_state(SlotState::Empty);
        // apply the gravity translation rustomino
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            log::debug!(
                "applying gravity: {:?} to {:?}",
                current_rustomino,
                current_rustomino.translated(TranslationDirection::DOWN_TRANSLATION),
            );
            current_rustomino.translate(TranslationDirection::DOWN_TRANSLATION);
        }
        self.set_rustomino_slot_state(SlotState::Occupied);
    }

    /// lock the current rustomino
    pub fn lock_rustomino(&mut self) {
        // get the current rustomino
        let Some(current_rustomino) = self.current_rustomino.as_mut() else {
            return;
        };

        log::debug!("locking rustomino: {:?}", current_rustomino);

        // locked the board slots to this rustomino type
        for slot in current_rustomino.board_slots() {
            // slots[y][x]
            self.slots[slot[1] as usize][slot[0] as usize] =
                SlotState::Locked(current_rustomino.rustomino_type);
        }

        // prepare for the next rustomino
        self.current_rustomino = None;
        self.update_ghost_rustomino();
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
        let num_completed_lines = completed_lines.len();
        if num_completed_lines == 0 {
            return completed_lines;
        }

        log::info!("clearing completed lines: {:?}", completed_lines);

        // iterate through the slots
        // set slots in the completed lines to empty
        let first_clear_line = completed_lines[0];
        let slots_before_clear = self.slots;
        for (y, slots_x) in self.slots.iter_mut().enumerate() {
            if y == 20 {
                break;
            }
            // clear the completed line
            if completed_lines.contains(&y) {
                for slot in slots_x.iter_mut() {
                    *slot = SlotState::Empty;
                }
            }
        }
        // then "move" the states of the slots above the cleared lines
        // down by the number of cleared lines
        for (y, slots_x) in self.slots.iter_mut().enumerate() {
            if y < first_clear_line {
                continue;
            }
            if y == PLAYFIELD_SIZE[1] as usize
                || y + num_completed_lines >= PLAYFIELD_SIZE[1] as usize
            {
                break;
            }
            for (x, slot) in slots_x.iter_mut().enumerate() {
                *slot = slots_before_clear[y + num_completed_lines as usize][x];
            }
        }
        self.update_ghost_rustomino();
        completed_lines
    }

    /// check to see if the provided block locations collide with other locked blocks
    /// or with walls
    pub fn check_collision(&self, block_locations: [IVec2; 4]) -> bool {
        log::debug!("check collision called: {:?}", block_locations);
        for location in block_locations {
            // check for left and right wall collisions
            if location[0] < 0 || location[0] >= BOARD_SLOTS[0] {
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

    /// Attempt to rotate the current rustomino
    pub fn rotate_rustomino(&mut self, direction: RotationDirection) -> bool {
        if let Some(current_rustomino) = &self.current_rustomino {
            // get the rustomino blocks if they were rotated
            let rotated_blocks = current_rustomino.rotated(&direction);

            // check to see if the translation would cause a collision with a locked block
            if self.check_collision(rotated_blocks) {
                log::debug!("rotation collision detected: {:?}", rotated_blocks);
                return false;
            }
        } else {
            return false;
        }

        self.set_rustomino_slot_state(SlotState::Empty);

        // get mutable reference
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // perform the tranlsation
            current_rustomino.rotate(&direction);
        }

        self.set_rustomino_slot_state(SlotState::Occupied);

        self.update_ghost_rustomino();

        true
    }

    /// Attempt to translate the current rustomino.
    /// Return true if possible
    pub fn translate_rustomino(&mut self, direction: TranslationDirection) -> bool {
        if let Some(current_rustomino) = &self.current_rustomino {
            let translated_blocks = current_rustomino.translated(direction.get_translation());

            // check to see if the translation would cause a collision with a locked block
            if self.check_collision(translated_blocks) {
                return false;
            }
        } else {
            return false;
        }

        self.set_rustomino_slot_state(SlotState::Empty);

        // get mutable reference
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // perform the tranlsation
            current_rustomino.translate(direction.get_translation());
        } else {
            return false;
        }

        self.set_rustomino_slot_state(SlotState::Occupied);

        self.update_ghost_rustomino();
        true
    }

    fn update_ghost_rustomino(&mut self) {
        let drop_translation = self.get_hard_drop_translation();
        if self.current_rustomino.is_some() {
            if let Some(ghost_rustomino) = &self.ghost_rustomino {
                self.set_slot_states(ghost_rustomino.board_slots(), SlotState::Empty);
            }
            if let Some(ghost_rustomino) = self.ghost_rustomino.as_mut() {
                ghost_rustomino.blocks = self.current_rustomino.as_ref().unwrap().blocks;
                ghost_rustomino.translation = self.current_rustomino.as_ref().unwrap().translation;
                ghost_rustomino.translate(drop_translation);
            }
            if let Some(ghost_rustomino) = &self.ghost_rustomino {
                self.set_slot_states(
                    ghost_rustomino.board_slots(),
                    SlotState::Ghost(ghost_rustomino.rustomino_type),
                );
            }
        } else {
            self.ghost_rustomino = None;
        }
    }

    fn set_rustomino_slot_state(&mut self, new_state: SlotState) {
        // safely unwrap current rustomino
        if let Some(current_rustomino) = &self.current_rustomino {
            self.set_slot_states(current_rustomino.board_slots(), new_state);
        }
    }

    pub fn hard_drop(&mut self) {
        let delta = self.get_hard_drop_translation();
        self.set_rustomino_slot_state(SlotState::Empty);
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            current_rustomino.translate(delta);
        }
    }

    fn get_hard_drop_translation(&self) -> IVec2 {
        if let Some(current_rustomino) = &self.current_rustomino {
            let mut translation = TranslationDirection::DOWN_TRANSLATION;

            // if we can't move it down without colliding the delta is 0
            if self.check_collision(current_rustomino.translated(translation)) {
                return IVec2::ZERO;
            }

            // keep attempting to move the rustomino down until it collides and return
            // the last non-coliding translation
            loop {
                let good_translation = translation;
                translation = translation + TranslationDirection::DOWN_TRANSLATION;
                if self.check_collision(current_rustomino.translated(translation)) {
                    return good_translation;
                }
            }
        }
        IVec2::ZERO
    }
}

// for debugging
impl Display for RustrisBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "-".repeat(BOARD_SLOTS[0] as usize * 2))?;
        for row in self.slots.iter().rev() {
            for slot in row {
                write!(f, "{}", slot)?;
            }
            writeln!(f)?;
        }
        write!(f, "{}", "-".repeat(BOARD_SLOTS[0] as usize * 2))?;
        Ok(())
    }
}
pub enum TranslationDirection {
    Left,
    Right,
    Down,
}

impl TranslationDirection {
    const LEFT_TRANSLATION: IVec2 = IVec2::new(-1, 0);
    const RIGHT_TRANSLATION: IVec2 = IVec2::new(1, 0);
    const DOWN_TRANSLATION: IVec2 = IVec2::new(0, -1);
    pub fn get_translation(&self) -> IVec2 {
        match self {
            TranslationDirection::Left => Self::LEFT_TRANSLATION,
            TranslationDirection::Right => Self::RIGHT_TRANSLATION,
            TranslationDirection::Down => Self::DOWN_TRANSLATION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Occupied,
    Locked(RustominoType),
    Ghost(RustominoType),
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SlotState::Empty => write!(f, "  ")?,
            SlotState::Occupied => write!(f, " #")?,
            SlotState::Locked(_) => write!(f, " @")?,
            SlotState::Ghost(_) => write!(f, " %")?,
        }
        Ok(())
    }
}
