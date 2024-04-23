use ggez::glam::IVec2;

use crate::{
    rustomino::{translated, Rotation, Rustomino, RustominoState, RustominoType},
    util::variants_equal,
};
use std::fmt::Display;

pub const PLAYFIELD_SLOTS: [usize; 2] = [10, 22];
pub const PLAYFIELD_SIZE: [i32; 2] = [10, 20];

type PlayfieldSlots = [[SlotState; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]];

#[derive(Debug)]
pub struct Playfield {
    pub slots: PlayfieldSlots,
    pub active_rustomino: Option<Rustomino>,
    pub ghost_rustomino: Option<Rustomino>,
}

impl Playfield {
    pub fn new() -> Self {
        log::info!("Initializing Playfield");
        Playfield {
            slots: [[SlotState::Empty; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]],
            active_rustomino: None,
            ghost_rustomino: None,
        }
    }

    /// Adds a new rustomino to the playfield
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn set_active(&mut self, rustomino: Rustomino) -> bool {
        log::info!("playing new rustomino: {:?}", rustomino.rtype);
        log::trace!("new rustomino: {:?}", rustomino);
        let ok = !check_collision(&self.slots, rustomino.playfield_slots());
        set_playfield_slot_states(
            &mut self.slots,
            &rustomino.playfield_slots(),
            SlotState::Occupied(rustomino.rtype),
        );
        self.ghost_rustomino = Some(rustomino.clone());
        self.active_rustomino = Some(rustomino);
        self.update_ghost_rustomino(false);
        ok
    }

    pub fn take_active(&mut self) -> Option<Rustomino> {
        let active_rustomino = self.active_rustomino.take()?;

        log::debug!("taking active rustomino: {:?}", active_rustomino.rtype);
        log::trace!("rustomino: {:?}", active_rustomino);
        set_playfield_slot_states(
            &mut self.slots,
            &active_rustomino.playfield_slots(),
            SlotState::Empty,
        );
        self.update_ghost_rustomino(false);
        Some(active_rustomino.reset())
    }
    /// checks to see if the playfield needs the next rustomino
    pub fn ready_for_next(&self) -> bool {
        self.active_rustomino.is_none()
    }

    // checking if rustomino can fall
    pub fn active_can_fall(&self) -> bool {
        log::debug!("checking if the active rustomino can fall");
        // get the active rustomino
        let Some(rustomino) = &self.active_rustomino else {
            return false;
        };

        // check if moving would cause a collision
        if check_collision(
            &self.slots,
            rustomino.translated(&TranslationDirection::DOWN_TRANSLATION),
        ) {
            return false;
        }

        true
    }

    pub fn get_active_state(&self) -> Option<RustominoState> {
        self.active_rustomino
            .as_ref()
            .map(|active_rustomino| active_rustomino.state)
    }

    pub fn set_active_state(&mut self, new_state: RustominoState) {
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            active_rustomino.set_state(new_state)
        }
    }

    /// Attempt to rotate the active rustomino
    pub fn rotate_active(&mut self, rotation: Rotation) -> bool {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return false;
        };

        // check to see if the block can be rotated with or without a wall kick
        let Some(wall_kick_trans) = check_rotation(&self.slots, active_rustomino, &rotation) else {
            return false;
        };

        // clear the current slot states
        set_playfield_slot_states(
            &mut self.slots,
            &active_rustomino.playfield_slots(),
            SlotState::Empty,
        );

        // perform the translation
        active_rustomino.rotate(&rotation, &wall_kick_trans);

        // set the new slot states to occupied
        set_playfield_slot_states(
            &mut self.slots,
            &active_rustomino.playfield_slots(),
            SlotState::Occupied(active_rustomino.rtype),
        );

        self.update_ghost_rustomino(true);

        true
    }

    /// Attempt to translate the active rustomino.
    /// Return true if possible
    pub fn translate_active(&mut self, direction: TranslationDirection) -> bool {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return false;
        };

        // check to see if the translation would cause a collision with a locked block
        let translated_blocks = active_rustomino.translated(&direction.get_translation());
        if check_collision(&self.slots, translated_blocks) {
            log::debug!("cannot translate, collision detected");
            return false;
        }

        translate_rustomino(
            &mut self.slots,
            SlotState::Occupied(active_rustomino.rtype),
            active_rustomino,
            direction.get_translation(),
        );

        self.update_ghost_rustomino(true);

        true
    }

    pub fn hard_drop_active(&mut self) {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return;
        };
        let delta = get_hard_drop_translation(&self.slots, active_rustomino);
        set_playfield_slot_states(
            &mut self.slots,
            &active_rustomino.playfield_slots(),
            SlotState::Empty,
        );
        active_rustomino.translate(delta);
    }

    /// lock the active rustomino
    pub fn lock_active(&mut self) {
        // get the active rustomino
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            log::info!("locking rustomino: {:?}", active_rustomino.rtype);
            log::trace!("rustomino: {:?}", active_rustomino);

            set_playfield_slot_states(
                &mut self.slots,
                &active_rustomino.playfield_slots(),
                SlotState::Locked(active_rustomino.rtype),
            );

            // prepare for the next rustomino
            self.active_rustomino = None;
            self.update_ghost_rustomino(true);
        }
    }

    /// apply gravity to the active rustomino
    pub fn apply_gravity(&mut self) {
        log::debug!("applying gravity");
        // apply the gravity translation rustomino
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            log::trace!(
                "applying gravity: {:?} to {:?}",
                active_rustomino,
                active_rustomino.translated(&TranslationDirection::DOWN_TRANSLATION),
            );
            translate_rustomino(
                &mut self.slots,
                SlotState::Occupied(active_rustomino.rtype),
                active_rustomino,
                TranslationDirection::Down.get_translation(),
            );
        }
    }

    pub fn clear_completed_lines(&mut self) -> Vec<usize> {
        let completed_lines = self.get_complete_lines();
        let num_completed_lines = completed_lines.len();
        if num_completed_lines == 0 {
            return completed_lines;
        }

        log::trace!("clearing lines before: playfield:\n{}", self);
        log::info!("clearing completed lines: {:?}", completed_lines);

        // iterate through the slots
        // clearing completed lines
        self.slots
            .iter_mut()
            .enumerate()
            .filter(|(y, _)| completed_lines.contains(y))
            .for_each(|(_, slots_x)| {
                for slot in slots_x.iter_mut() {
                    *slot = SlotState::Empty;
                }
            });

        log::trace!("clearing lines middle: playfield:\n{}", self);

        // then "move" the states of the slots above cleared lines down
        // starts at the highest cleared line, and moves block states down
        // this can probably be improved
        for line in completed_lines.iter().rev() {
            for y in *line..self.slots.len() {
                for x in 0..self.slots[y].len() {
                    // is this line is the very top row
                    if y + 1 >= PLAYFIELD_SLOTS[1] {
                        self.slots[y][x] = SlotState::Empty; // set all slots to empty
                    } else {
                        self.slots[y][x] = self.slots[y + 1][x]; // cope from the line above
                    }
                }
            }
        }

        log::trace!("clearing lines after: playfield:\n{}", self);
        self.update_ghost_rustomino(false);
        completed_lines
    }

    /// Returns the get complete lines of this [`Playfield`].
    fn get_complete_lines(&self) -> Vec<usize> {
        let mut complete_lines = vec![];
        'outer: for (i, line) in self.slots.iter().enumerate() {
            for slot in line {
                // compare variant ignoring the value
                if !variants_equal(slot, &SlotState::Locked(RustominoType::I)) {
                    continue 'outer;
                }
            }
            complete_lines.push(i);
        }
        complete_lines
    }

    fn update_ghost_rustomino(&mut self, translating: bool) {
        let Some(active_rustomino) = &self.active_rustomino else {
            log::debug!("active_rustomino is None, removing ghost rustomino");
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
            return;
        };

        log::debug!("updating ghost location");
        let drop_translation = get_hard_drop_translation(&self.slots, active_rustomino);
        if let Some(ghost_rustomino) = self.ghost_rustomino.as_mut() {
            if translating {
                for slot in ghost_rustomino.playfield_slots() {
                    if !variants_equal(
                        &self.slots[slot[1] as usize][slot[0] as usize],
                        &SlotState::Occupied(RustominoType::I),
                    ) {
                        self.slots[slot[1] as usize][slot[0] as usize] = SlotState::Empty;
                    }
                }
            }

            ghost_rustomino.blocks = active_rustomino.blocks;
            ghost_rustomino.translation = active_rustomino.translation;

            // perform the translation
            ghost_rustomino.translate(drop_translation);

            log::trace!(
                "update_ghost_rustomino: new ghost rustomino location: {:?}",
                ghost_rustomino.playfield_slots()
            );

            // set the new slot states to occupied
            for slot in ghost_rustomino.playfield_slots() {
                if !variants_equal(
                    &self.slots[slot[1] as usize][slot[0] as usize],
                    &SlotState::Occupied(RustominoType::I),
                ) {
                    self.slots[slot[1] as usize][slot[0] as usize] =
                        SlotState::Ghost(ghost_rustomino.rtype);
                }
            }
        }
    }
}

fn get_hard_drop_translation(playfield_slots: &PlayfieldSlots, rustomino: &Rustomino) -> IVec2 {
    let mut translation = TranslationDirection::DOWN_TRANSLATION;

    // if we can't move it down without colliding the delta is 0
    if check_collision(playfield_slots, rustomino.translated(&translation)) {
        log::debug!("hard_drop_translation: cannot move, block on stack");
        return IVec2::ZERO;
    }

    // keep attempting to move the rustomino down until it collides and return
    // the last non-colliding translation
    loop {
        let good_translation = translation;
        translation += TranslationDirection::DOWN_TRANSLATION;
        if check_collision(playfield_slots, rustomino.translated(&translation)) {
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
fn check_collision(playfield_slots: &PlayfieldSlots, block_locations: [IVec2; 4]) -> bool {
    for location in block_locations {
        // check for left and right wall collisions
        if location[0] < 0 || location[0] >= PLAYFIELD_SLOTS[0] as i32 {
            log::trace!("collided with left/right wall: {:?}", block_locations);
            return true;
        }
        if location[1] >= PLAYFIELD_SLOTS[1] as i32 {
            log::trace!("collided with top wall: {:?}", block_locations);
            return true;
        }
        // check for bottom wall collision
        if location[1] < 0 {
            log::trace!("collided with bottom wall: {:?}", block_locations);
            return true;
        }
        // slots[y][x] compare variant ignoring value
        if variants_equal(
            &playfield_slots[location[1] as usize][location[0] as usize],
            &SlotState::Locked(RustominoType::I),
        ) {
            log::trace!("collided with locked block: {:?}", block_locations);
            return true;
        }
    }
    false
}

fn check_rotation(
    playfield_slots: &PlayfieldSlots,
    rustomino: &Rustomino,
    rotation: &Rotation,
) -> Option<IVec2> {
    let wall_kick_tests = rustomino.wall_kick_tests(rotation);
    let rotated_blocks = rustomino.rotated(rotation);
    wall_kick_tests
        .iter()
        .find(|x| !check_collision(playfield_slots, translated(&rotated_blocks, x)))
        .copied()
}

fn translate_rustomino(
    playfield_slots: &mut PlayfieldSlots,
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
    // perform the translation
    rustomino.translate(translation);
    // set the new slot states to occupied
    set_playfield_slot_states(playfield_slots, &rustomino.playfield_slots(), new_state);
}

fn set_playfield_slot_states(
    playfield_slots: &mut PlayfieldSlots,
    block_slots: &[IVec2; 4],
    new_state: SlotState,
) {
    log::info!(
        "set_slot_state called block_slots: {:?} to state: {:?}",
        block_slots,
        new_state
    );
    for slot in block_slots {
        playfield_slots[slot[1] as usize][slot[0] as usize] = new_state;
    }
}

// display the playfield's slot states for debugging
impl Display for Playfield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (y, row) in self.slots.iter().rev().enumerate() {
            if y == 2 {
                writeln!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
            }
            for slot in row {
                write!(f, "{slot}")?;
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
