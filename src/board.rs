use crate::rustomino::{RotationDirection, Rustomino, RustominoType};
use piston_window::types::Vec2d;
use std::fmt::Display;
use std::mem::discriminant;

// playfield is 10 rows wide, 20 columns high
// new rustomino's are spawned above playfield in lines 21,22
pub const SLOTS_AREA: [usize; 2] = [10, 22];
pub const PLAYFIELD_SIZE: [usize; 2] = [10, 20];
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

// RustrisBoard
#[derive(Debug)]
pub struct RustrisBoard {
    pub(crate) slots: [[SlotState; SLOTS_AREA[0]]; SLOTS_AREA[1]],
    pub(crate) current_rustomino: Option<Rustomino>,
    pub(crate) ghost_rustomino: Option<Rustomino>,
}

impl Display for RustrisBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "-".repeat(SLOTS_AREA[0] * 2))?;
        for row in self.slots.iter().rev() {
            for slot in row {
                write!(f, "{}", slot)?;
            }
            writeln!(f)?;
        }
        write!(f, "{}", "-".repeat(SLOTS_AREA[0] * 2))?;
        Ok(())
    }
}

impl RustrisBoard {
    pub fn new() -> Self {
        log::info!("Initializing Rustris Board");
        RustrisBoard {
            slots: [[SlotState::Empty; SLOTS_AREA[0]]; SLOTS_AREA[1]],
            current_rustomino: None,
            ghost_rustomino: None,
        }
    }

    // Add the rustomino to the board
    // returns false if there was a collision while adding the block (game over)
    pub fn add_new_rustomino(&mut self, rustomino: Rustomino) -> bool {
        let ok = !self.check_collision(rustomino.board_slots());
        // slots[y][x]
        self.set_slot_state(rustomino.board_slots(), SlotState::Occupied);
        self.ghost_rustomino = Some(rustomino.clone());
        self.current_rustomino = Some(rustomino);
        self.update_ghost_rustomino();
        ok
    }

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

    pub fn apply_gravity(&mut self) {
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
            if y == PLAYFIELD_SIZE[1] || y + num_completed_lines >= PLAYFIELD_SIZE[1] {
                break;
            }
            for (x, slot) in slots_x.iter_mut().enumerate() {
                *slot = slots_before_clear[y + num_completed_lines][x];
            }
        }
        self.update_ghost_rustomino();
        completed_lines
    }

    /// check to see if the provided block locations collide with other locked blocks
    /// or with walls
    pub fn check_collision(&self, block_locations: [Vec2d<i32>; 4]) -> bool {
        log::debug!("check collision called: {:?}", block_locations);
        for location in block_locations {
            // check for left and right wall collisions
            if location[0] < 0 || location[0] >= SLOTS_AREA[0] as i32 {
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

    pub fn hard_drop(&mut self) {
        let delta = self.drop_translation();
        self.set_current_rustomino_slot_state(SlotState::Empty);
        self.current_rustomino.as_mut().unwrap().translate(delta);
        self.lock_rustomino();
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

        self.update_ghost_rustomino();
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

        self.update_ghost_rustomino();
        true
    }

    fn update_ghost_rustomino(&mut self) {
        let drop_translation = self.drop_translation();
        if self.current_rustomino.is_some() {
            if let Some(ghost_rustomino) = &self.ghost_rustomino {
                self.set_slot_state(ghost_rustomino.board_slots(), SlotState::Empty);
            }
            if let Some(ghost_rustomino) = self.ghost_rustomino.as_mut() {
                ghost_rustomino.blocks = self.current_rustomino.as_ref().unwrap().blocks;
                ghost_rustomino.translation = self.current_rustomino.as_ref().unwrap().translation;
                ghost_rustomino.translate(drop_translation);
            }
            if let Some(ghost_rustomino) = &self.ghost_rustomino {
                self.set_slot_state(
                    ghost_rustomino.board_slots(),
                    SlotState::Ghost(ghost_rustomino.rustomino_type),
                );
            }
        } else {
            // if let Some(ghost_rustomino) = &self.ghost_rustomino {
            //     self.set_slot_state(ghost_rustomino.block_slots(), SlotState::Empty);
            // }
            self.ghost_rustomino = None;
        }
    }

    fn set_current_rustomino_slot_state(&mut self, new_state: SlotState) {
        // safely unwrap current rustomino
        if let Some(current_rustomino) = &self.current_rustomino {
            self.set_slot_state(current_rustomino.board_slots(), new_state);
        }
    }

    fn drop_translation(&self) -> Vec2d<i32> {
        if let Some(current_rustomino) = &self.current_rustomino {
            let mut translation = GRAVITY_TRANSLATION;

            // if we can't move it down without colliding the delta is 0
            if self.check_collision(current_rustomino.translated(translation)) {
                return [0, 0];
            }

            // keep attempting to move the rustomino down until it collides and return
            // the last non-coliding translation
            loop {
                let good_translation = translation;
                translation = vecmath::vec2_add(translation, [0, -1]);
                if self.check_collision(current_rustomino.translated(translation)) {
                    return good_translation;
                }
            }
        }
        [0, 0]
    }
}
