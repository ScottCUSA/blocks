use std::{fmt::Display, mem::discriminant};

use macroquad::prelude::*;

use crate::rustomino::{RotationDirection, Rustomino, RustominoType};

pub(crate) const PLAYFIELD_SLOTS: [usize; 2] = [10, 22];
pub(crate) const PLAYFIELD_SIZE: [i32; 2] = [10, 20];

type Slots = [[SlotState; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]];

// RustrisPlayfield
#[derive(Debug)]
pub struct RustrisPlayfield {
    pub(crate) slots: Slots,
    pub(crate) current_rustomino: Option<Rustomino>,
    pub(crate) ghost_rustomino: Option<Rustomino>,
}

impl RustrisPlayfield {
    pub fn new() -> Self {
        log::info!("Initializing Rustris Playfield");
        RustrisPlayfield {
            slots: [[SlotState::Empty; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]],
            current_rustomino: None,
            ghost_rustomino: None,
        }
    }

    /// Adds a new rustomino to the playfield
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn set_current_rustomino(&mut self, rustomino: Rustomino) -> bool {
        log::debug!("setting current rustomino: {:?}", rustomino);
        let ok = !check_collision(&self.slots, rustomino.playfield_slots());
        set_playfield_slot_states(
            &mut self.slots,
            &rustomino.playfield_slots(),
            SlotState::Occupied(rustomino.rustomino_type),
        );
        self.ghost_rustomino = Some(rustomino.clone());
        self.current_rustomino = Some(rustomino);
        self.update_ghost_rustomino(false);
        ok
    }

    /// Adds a new rustomino to the playfield
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn take_current(&mut self) -> Option<Rustomino> {
        if let Some(current_rustomino) = self.current_rustomino.take() {
            log::debug!("taking current rustomino: {:?}", current_rustomino);
            set_playfield_slot_states(
                &mut self.slots,
                &current_rustomino.playfield_slots(),
                SlotState::Empty,
            );
            self.update_ghost_rustomino(false);
            return Some(current_rustomino.reset());
        }
        None
    }
    /// checks to see if the playfield needs the next rustomino
    pub fn ready_for_next(&self) -> bool {
        self.current_rustomino.is_none()
    }

    // checking if rustomino can fall
    pub fn can_fall(&self) -> bool {
        log::debug!("checking if the current rustomino can fall");
        // get the current rustomino
        let Some(rustomino) = &self.current_rustomino else {
            // no blocks to move/or lock
            return false;
        };

        // check if moving would cause a collision
        if check_collision(
            &self.slots,
            rustomino.translated(TranslationDirection::DOWN_TRANSLATION),
        ) {
            return false;
        }

        true
    }

    /// apply gravity to the current rustomino
    pub fn apply_gravity(&mut self) {
        // apply the gravity translation rustomino
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            log::debug!(
                "applying gravity: {:?} to {:?}",
                current_rustomino,
                current_rustomino.translated(TranslationDirection::DOWN_TRANSLATION),
            );
            translate_rustomino(
                &mut self.slots,
                SlotState::Occupied(current_rustomino.rustomino_type),
                current_rustomino,
                TranslationDirection::Down.get_translation(),
            );
        }
    }

    /// lock the current rustomino
    pub fn lock_rustomino(&mut self) {
        // get the current rustomino
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            log::debug!("locking rustomino: {:?}", current_rustomino);

            set_playfield_slot_states(
                &mut self.slots,
                &current_rustomino.playfield_slots(),
                SlotState::Locked(current_rustomino.rustomino_type),
            );

            // prepare for the next rustomino
            self.current_rustomino = None;
            self.update_ghost_rustomino(true);
        }
    }

    /// Returns the get complete lines of this [`RustrisPlayfield`].
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

        log::debug!("clearing lines before: playfield:\n{}", self);

        log::info!("clearing completed lines: {:?}", completed_lines);

        // iterate through the slots
        // skip to the lowest completed line
        let lowest_completed_line = completed_lines[0];
        let slots_before_clear = self.slots;
        for (y, slots_x) in self
            .slots
            .iter_mut()
            .enumerate()
            .skip(lowest_completed_line)
        {
            // clear the completed line
            if completed_lines.contains(&y) {
                for slot in slots_x.iter_mut() {
                    *slot = SlotState::Empty;
                }
            }
        }

        log::debug!("clearing lines middle: playfield:\n{}", self);
        // then "move" the states of the slots above cleared lines
        // down by the number of cleared lines
        // start at the lowest completed line
        for (y, slots_x) in self
            .slots
            .iter_mut()
            .enumerate()
            .skip(lowest_completed_line)
        {
            // can't shift rows that don't exist down
            if y + num_completed_lines >= PLAYFIELD_SLOTS[1] {
                break;
            }
            for (x, slot) in slots_x.iter_mut().enumerate() {
                *slot = slots_before_clear[y + num_completed_lines][x];
            }
        }
        log::debug!("clearing lines after: playfield:\n{}", self);
        self.update_ghost_rustomino(false);
        completed_lines
    }

    /// Attempt to rotate the current rustomino
    pub fn rotate_current(&mut self, direction: RotationDirection) -> bool {
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // get the rustomino blocks if they were rotated
            let rotated_blocks = current_rustomino.rotated(&direction);

            // check to see if the translation would cause a collision with a locked block
            if check_collision(&self.slots, rotated_blocks) {
                log::debug!("rotation collision detected: {:?}", rotated_blocks);
                return false;
            }

            rotate_rustomino(
                &mut self.slots,
                SlotState::Occupied(current_rustomino.rustomino_type),
                current_rustomino,
                &direction,
            );

            self.update_ghost_rustomino(true);
        } else {
            return false;
        }
        true
    }

    /// Attempt to translate the current rustomino.
    /// Return true if possible
    pub fn translate_current(&mut self, direction: TranslationDirection) -> bool {
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            // check to see if the translation would cause a collision with a locked block
            let translated_blocks = current_rustomino.translated(direction.get_translation());
            if check_collision(&self.slots, translated_blocks) {
                log::debug!("translate collision detected: {:?}", translated_blocks);
                return false;
            }

            translate_rustomino(
                &mut self.slots,
                SlotState::Occupied(current_rustomino.rustomino_type),
                current_rustomino,
                direction.get_translation(),
            );

            self.update_ghost_rustomino(true);
        } else {
            return false;
        }

        true
    }

    pub fn update_ghost_rustomino(&mut self, translating: bool) {
        if let Some(current_rustomino) = &self.current_rustomino {
            log::debug!("update_ghost_rustomino: updating ghost location");
            let drop_translation = get_hard_drop_translation(&self.slots, current_rustomino);
            if let Some(ghost_rustomino) = self.ghost_rustomino.as_mut() {
                if translating {
                    for slot in ghost_rustomino.playfield_slots() {
                        if discriminant(&self.slots[slot[1] as usize][slot[0] as usize])
                            != discriminant(&SlotState::Occupied(RustominoType::I))
                        {
                            self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Empty;
                        }
                    }
                }

                ghost_rustomino.blocks = current_rustomino.blocks;
                ghost_rustomino.translation = current_rustomino.translation;

                // perform the tranlsation
                ghost_rustomino.translate(drop_translation);

                log::debug!(
                    "update_ghost_rustomino: new ghost rustomino location: {:?}",
                    ghost_rustomino.playfield_slots()
                );

                // set the new slot states to occupied
                for slot in ghost_rustomino.playfield_slots() {
                    if discriminant(&self.slots[slot[1] as usize][slot[0] as usize])
                        != discriminant(&SlotState::Occupied(RustominoType::I))
                    {
                        self.slots[slot[1] as usize][slot[0] as usize] =
                            SlotState::Ghost(ghost_rustomino.rustomino_type);
                    }
                }
            }
        } else {
            log::debug!("update_ghost_rustomino: removing ghost rustomino");
            if !translating {
                if let Some(ghost_rustomino) = self.ghost_rustomino.as_mut() {
                    set_playfield_slot_states(
                        &mut self.slots,
                        &ghost_rustomino.playfield_slots(),
                        SlotState::Empty,
                    );
                }
            }
            self.ghost_rustomino = None;
        }
    }

    pub fn hard_drop(&mut self) {
        if let Some(current_rustomino) = self.current_rustomino.as_mut() {
            let delta = get_hard_drop_translation(&self.slots, current_rustomino);
            set_playfield_slot_states(
                &mut self.slots,
                &current_rustomino.playfield_slots(),
                SlotState::Empty,
            );
            current_rustomino.translate(delta);
        }
    }
}

fn get_hard_drop_translation(playfield_slots: &Slots, rustomino: &Rustomino) -> IVec2 {
    let mut translation = TranslationDirection::DOWN_TRANSLATION;

    // if we can't move it down without colliding the delta is 0
    if check_collision(playfield_slots, rustomino.translated(translation)) {
        log::debug!("hard_drop_translation: cannot move, block on stack");
        return IVec2::ZERO;
    }

    // keep attempting to move the rustomino down until it collides and return
    // the last non-coliding translation
    loop {
        let good_translation = translation;
        translation += TranslationDirection::DOWN_TRANSLATION;
        if check_collision(playfield_slots, rustomino.translated(translation)) {
            log::debug!(
                "hard_drop_translation: found hard drop translation: {:?}",
                good_translation
            );
            return good_translation;
        }
    }
}

/// check to see if the provided block locations collide with other locked blocks
/// or with walls
fn check_collision(playfield_slots: &Slots, block_locations: [IVec2; 4]) -> bool {
    for location in block_locations {
        // check for left and right wall collisions
        if location[0] < 0 || location[0] >= PLAYFIELD_SLOTS[0] as i32 {
            log::debug!("collided with left/right wall: {:?}", block_locations);
            return true;
        }
        // check for bottom wall collision
        if location[1] < 0 {
            log::debug!("collided with bottom wall: {:?}", block_locations);
            return true;
        }
        // slots[y][x] compare variant ignoring value
        if discriminant(&playfield_slots[location[1] as usize][location[0] as usize])
            == discriminant(&SlotState::Locked(RustominoType::I))
        {
            log::debug!("collided with locked block: {:?}", block_locations);
            return true;
        }
    }
    false
}

fn translate_rustomino(
    playfield_slots: &mut Slots,
    new_state: SlotState,
    rustomino: &mut Rustomino,
    translation: IVec2,
) {
    // clear the current slot states
    set_playfield_slot_states(
        playfield_slots,
        &rustomino.playfield_slots(),
        SlotState::Empty,
    );
    // perform the tranlsation
    rustomino.translate(translation);
    // set the new slot states to occupied
    set_playfield_slot_states(playfield_slots, &rustomino.playfield_slots(), new_state);
}

fn rotate_rustomino(
    playfield_slots: &mut Slots,
    new_state: SlotState,
    rustomino: &mut Rustomino,
    rotation: &RotationDirection,
) {
    // clear the current slot states
    set_playfield_slot_states(
        playfield_slots,
        &rustomino.playfield_slots(),
        SlotState::Empty,
    );
    // perform the tranlsation
    rustomino.rotate(rotation);
    // set the new slot states to occupied
    set_playfield_slot_states(playfield_slots, &rustomino.playfield_slots(), new_state);
}

fn set_playfield_slot_states(
    playfield_slots: &mut Slots,
    block_slots: &[IVec2; 4],
    new_state: SlotState,
) {
    log::debug!(
        "set_slot_state called block_slots: {:?} to state: {:?}",
        block_slots,
        new_state
    );
    for slot in block_slots {
        playfield_slots[slot[1] as usize][slot[0] as usize] = new_state;
    }
}

// display the playfield's slot states for debugging
impl Display for RustrisPlayfield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
        for row in self.slots.iter().rev() {
            for slot in row {
                write!(f, "{}", slot)?;
            }
            writeln!(f)?;
        }
        write!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
        Ok(())
    }
}
#[derive(Debug)]
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
    Occupied(RustominoType),
    Locked(RustominoType),
    Ghost(RustominoType),
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SlotState::Empty => write!(f, "  ")?,
            SlotState::Occupied(_) => write!(f, " #")?,
            SlotState::Locked(_) => write!(f, " @")?,
            SlotState::Ghost(_) => write!(f, " %")?,
        }
        Ok(())
    }
}
