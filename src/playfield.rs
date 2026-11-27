use ggez::glam::IVec2;

use crate::rustomino::{translated, Rotation, Rustomino, RustominoState, RustominoType};
use std::fmt::Display;

pub const PLAYFIELD_SLOTS: [usize; 2] = [10, 22];
pub const LINE_CLEAR_FADE: f64 = 0.5; // fraction of the line clear each block spends fading
pub const PLAYFIELD_SIZE: [i32; 2] = [10, 20];

pub type PlayfieldSlots = [[SlotState; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]];

#[derive(Debug)]
pub struct LineClear {
    pub lines: Vec<usize>,
    pub origins: Vec<usize>, // columns where the locked rustomino landed
    pub time: f64,           // seconds since the lines were completed
    pub duration: f64,       // seconds until the lines are removed
}

impl LineClear {
    /// fraction of the line clear completed, 0.0 to 1.0
    pub fn progress(&self) -> f64 {
        (self.time / self.duration).clamp(0.0, 1.0)
    }

    /// columns between x and the nearest origin
    pub fn distance(&self, x: usize) -> usize {
        self.origins
            .iter()
            .map(|origin| origin.abs_diff(x))
            .min()
            .unwrap_or(0)
    }

    /// nearest origin to x
    pub fn origin(&self, x: usize) -> usize {
        self.origins
            .iter()
            .copied()
            .min_by_key(|origin| origin.abs_diff(x))
            .unwrap_or(x)
    }

    /// fraction of the line clear when block x starts fading
    pub fn wipe_start(&self, x: usize) -> f64 {
        let max_distance = self.max_distance().max(1) as f64;
        self.distance(x) as f64 / max_distance * (1.0 - LINE_CLEAR_FADE)
    }

    pub fn max_distance(&self) -> usize {
        (0..PLAYFIELD_SLOTS[0])
            .map(|x| self.distance(x))
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct Playfield {
    pub slots: PlayfieldSlots, // locked blocks only
    pub active_rustomino: Option<Rustomino>,
    pub ghost_rustomino: Option<Rustomino>,
    pub line_clear: Option<LineClear>,
}

impl Playfield {
    pub fn new() -> Self {
        log::info!("Initializing Playfield");
        Playfield {
            slots: [[SlotState::Empty; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]],
            active_rustomino: None,
            ghost_rustomino: None,
            line_clear: None,
        }
    }

    /// Adds a new rustomino to the playfield
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn set_active(&mut self, rustomino: Rustomino) -> bool {
        log::info!("playing new rustomino: {:?}", rustomino.rtype);
        log::trace!("new rustomino: {:?}", rustomino);
        let ok = !check_collision(&self.slots, rustomino.playfield_slots());
        self.active_rustomino = Some(rustomino);
        self.update_ghost_rustomino();
        ok
    }

    pub fn take_active(&mut self) -> Option<Rustomino> {
        let active_rustomino = self.active_rustomino.take()?;

        log::debug!("taking active rustomino: {:?}", active_rustomino.rtype);
        log::trace!("rustomino: {:?}", active_rustomino);
        self.update_ghost_rustomino();
        Some(active_rustomino.reset())
    }

    /// checks to see if the playfield needs the next rustomino
    pub fn ready_for_next(&self) -> bool {
        self.active_rustomino.is_none()
    }

    // checking if rustomino can fall
    pub fn active_can_fall(&self) -> bool {
        self.active_rustomino.as_ref().is_some_and(|rustomino| {
            !check_collision(&self.slots, rustomino.translated(&IVec2::NEG_Y))
        })
    }

    /// lowest row occupied by the active rustomino
    pub fn active_lowest_row(&self) -> Option<i32> {
        self.active_rustomino
            .as_ref()
            .and_then(|r| r.playfield_slots().iter().map(|s| s.y).min())
    }

    pub fn get_active_state(&self) -> Option<RustominoState> {
        self.active_rustomino
            .as_ref()
            .map(|active_rustomino| active_rustomino.state)
    }

    pub fn set_active_state(&mut self, new_state: RustominoState) {
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            active_rustomino.state = new_state;
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

        active_rustomino.rotate(&rotation, &wall_kick_trans);
        self.update_ghost_rustomino();
        true
    }

    /// Attempt to translate the active rustomino.
    /// Return true if possible
    pub fn translate_active(&mut self, direction: TranslationDirection) -> bool {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return false;
        };

        // check to see if the translation would cause a collision with a locked block
        let translation = direction.get_translation();
        if check_collision(&self.slots, active_rustomino.translated(&translation)) {
            log::debug!("cannot translate, collision detected");
            return false;
        }

        active_rustomino.translate(translation);
        self.update_ghost_rustomino();
        true
    }

    /// returns the slots it dropped from
    pub fn hard_drop_active(&mut self) -> Option<[IVec2; 4]> {
        let active_rustomino = self.active_rustomino.as_mut()?;
        let from = active_rustomino.playfield_slots();
        let delta = get_hard_drop_translation(&self.slots, active_rustomino);
        active_rustomino.translate(delta);
        Some(from)
    }

    /// lock the active rustomino
    /// returns the slots it was locked into
    pub fn lock_active(&mut self) -> Option<[IVec2; 4]> {
        let active_rustomino = self.active_rustomino.take()?;
        log::info!("locking rustomino: {:?}", active_rustomino.rtype);
        log::trace!("rustomino: {:?}", active_rustomino);

        let slots = active_rustomino.playfield_slots();
        for slot in slots {
            self.slots[slot.y as usize][slot.x as usize] =
                SlotState::Locked(active_rustomino.rtype);
        }
        self.update_ghost_rustomino();
        Some(slots)
    }

    /// apply gravity to the active rustomino
    pub fn apply_gravity(&mut self) {
        log::trace!("applying gravity");
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            active_rustomino.translate(IVec2::NEG_Y);
        }
    }

    /// start clearing any completed lines
    /// returns the number of lines being cleared
    pub fn start_line_clear(&mut self, duration: f64, landed: &[IVec2]) -> usize {
        let lines = complete_lines(&self.slots);
        if lines.is_empty() {
            return 0;
        }
        log::info!("clearing completed lines: {:?}", lines);
        let num_lines = lines.len();
        let mut origins: Vec<usize> = landed.iter().map(|slot| slot.x as usize).collect();
        origins.sort_unstable();
        origins.dedup();
        self.line_clear = Some(LineClear {
            lines,
            origins,
            time: 0.0,
            duration,
        });
        num_lines
    }

    pub fn is_clearing_lines(&self) -> bool {
        self.line_clear.is_some()
    }

    /// advance the line clear, removing the lines when it finishes
    pub fn update_line_clear(&mut self, delta_time: f64) {
        let Some(line_clear) = self.line_clear.as_mut() else {
            return;
        };
        line_clear.time += delta_time;
        if line_clear.time >= line_clear.duration {
            let line_clear = self.line_clear.take().expect("line clear in progress");
            remove_lines(&mut self.slots, &line_clear.lines);
        }
    }

    fn update_ghost_rustomino(&mut self) {
        self.ghost_rustomino = self.active_rustomino.as_ref().map(|active_rustomino| {
            let mut ghost_rustomino = active_rustomino.clone();
            ghost_rustomino.translate(get_hard_drop_translation(&self.slots, active_rustomino));
            ghost_rustomino
        });
    }
}

fn get_hard_drop_translation(playfield_slots: &PlayfieldSlots, rustomino: &Rustomino) -> IVec2 {
    let mut translation = IVec2::ZERO;
    while !check_collision(
        playfield_slots,
        rustomino.translated(&(translation + IVec2::NEG_Y)),
    ) {
        translation += IVec2::NEG_Y;
    }
    translation
}

/// blocks outside the walls or on locked blocks
pub fn check_collision(slots: &PlayfieldSlots, blocks: [IVec2; 4]) -> bool {
    blocks.iter().any(|b| {
        !(0..PLAYFIELD_SLOTS[0] as i32).contains(&b.x)
            || !(0..PLAYFIELD_SLOTS[1] as i32).contains(&b.y)
            || slots[b.y as usize][b.x as usize] != SlotState::Empty
    })
}

/// the first wall kick that lets the rustomino rotate
pub fn check_rotation(
    slots: &PlayfieldSlots,
    rustomino: &Rustomino,
    rotation: &Rotation,
) -> Option<IVec2> {
    let rotated = rustomino.rotated(rotation);
    rustomino
        .wall_kick_tests(rotation)
        .into_iter()
        .find(|kick| !check_collision(slots, translated(&rotated, kick)))
}

pub fn complete_lines(slots: &PlayfieldSlots) -> Vec<usize> {
    (0..slots.len())
        .filter(|&y| slots[y].iter().all(|slot| *slot != SlotState::Empty))
        .collect()
}

/// move the remaining lines down in one pass
pub fn remove_lines(slots: &mut PlayfieldSlots, lines: &[usize]) {
    let mut write = 0;
    for read in 0..slots.len() {
        if !lines.contains(&read) {
            slots[write] = slots[read];
            write += 1;
        }
    }
    for line in &mut slots[write..] {
        *line = [SlotState::Empty; PLAYFIELD_SLOTS[0]];
    }
}

// display the playfield's slot states for debugging
impl Display for Playfield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active = self.active_rustomino.as_ref().map(|r| r.playfield_slots());
        let ghost = self.ghost_rustomino.as_ref().map(|r| r.playfield_slots());
        let contains = |blocks: &Option<[IVec2; 4]>, slot: IVec2| {
            blocks.is_some_and(|blocks| blocks.contains(&slot))
        };
        for (y, row) in self.slots.iter().enumerate().rev() {
            if y + 1 == PLAYFIELD_SIZE[1] as usize {
                writeln!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
            }
            for (x, slot) in row.iter().enumerate() {
                let pos = IVec2::new(x as i32, y as i32);
                if contains(&active, pos) {
                    write!(f, " #")?;
                } else if contains(&ghost, pos) {
                    write!(f, " %")?;
                } else {
                    write!(f, "{slot}")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TranslationDirection {
    Left,
    Right,
    Down,
}

impl TranslationDirection {
    pub fn get_translation(&self) -> IVec2 {
        match self {
            TranslationDirection::Left => IVec2::NEG_X,
            TranslationDirection::Right => IVec2::X,
            TranslationDirection::Down => IVec2::NEG_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Locked(RustominoType),
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if *self == SlotState::Empty {
            "  "
        } else {
            " @"
        };
        write!(f, "{text}")
    }
}
