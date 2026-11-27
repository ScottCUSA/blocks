//! attract mode: idle at the menu, the computer plays a demo game, then the high scores

use ggez::glam::IVec2;
use rand::RngExt;
use rand_xoshiro::Xoshiro256PlusPlus;

use super::{gravity_delay, BlocksState, GameState, LINES_PER_LEVEL};
use crate::effects::Effects;
use crate::level_progress::LevelProgress;
use crate::playfield::{
    check_collision, check_rotation, complete_lines, remove_lines, PlayfieldSlots, SlotState,
    TranslationDirection, PLAYFIELD_SIZE, PLAYFIELD_SLOTS,
};
use crate::rustomino::{Rotation, Rustomino};
use crate::score_display::ScoreDisplay;

const IDLE_TIME: f64 = 30.0; // at the menu before a demo
pub(super) const DEMO_TIME: f64 = 30.0; // longest demo game
const GAME_OVER_TIME: f64 = 3.0; // game over shown, once the overlay is in
const SCORES_TIME: f64 = 10.0; // high scores shown after a demo
const MOVE_TIME: f64 = 0.11; // between moves, at slow gravity
const THINK_TIME: f64 = 0.25; // before the first move
pub(super) const DEMO_LEVEL: usize = 7;
/// by lines cleared, toward bigger clears
const CLEAR_BONUS: [f64; 5] = [0.0, -60.0, 0.0, 150.0, 400.0];
const WELL_FORGIVEN: u32 = 10; // 1 + 2 + 3 + 4
const SLIP_CHANCE: f64 = 0.1; // of a worse placement
const SLIP_RANK: usize = 3; // worst placement ranked from the best

const W: usize = PLAYFIELD_SLOTS[0];
const H: usize = PLAYFIELD_SLOTS[1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Off,
    Demo,
    Scores,
}

/// a placement: quarter turns (negative counterclockwise), then the column
#[derive(Debug, Clone, Copy, PartialEq)]
struct Plan {
    turns: i32,
    x: i32,
}

pub struct Attract {
    pub phase: Phase,
    idle: f64,
    timer: f64,
    plan: Option<Plan>,
    piece_y: i32, // the planned piece's row, a higher one is a new piece
    wait: f64,
    rng: Xoshiro256PlusPlus,
}

impl Attract {
    pub fn new() -> Self {
        Attract {
            phase: Phase::Off,
            idle: 0.0,
            timer: 0.0,
            plan: None,
            piece_y: 0,
            wait: 0.0,
            rng: rand::make_rng(),
        }
    }

    pub fn demo(&self) -> bool {
        self.phase == Phase::Demo
    }

    /// back to idle, keeping the rng
    pub(super) fn reset(&mut self, phase: Phase) {
        *self = Attract {
            phase,
            rng: self.rng.clone(),
            ..Attract::new()
        };
    }
}

impl BlocksState {
    /// start a demo when idle, end it on time or game over, then the scores
    pub(super) fn attract_update(&mut self, delta_time: f64) {
        match self.attract.phase {
            Phase::Off => {
                if self.state == GameState::Menu {
                    self.attract.idle += delta_time;
                } else {
                    self.attract.idle = 0.0;
                }
                if self.attract.idle >= IDLE_TIME {
                    self.start_demo();
                }
            }
            Phase::Demo => {
                self.attract.timer += delta_time;
                let over = match self.state {
                    GameState::Playing => {
                        self.computer(delta_time);
                        self.attract.timer >= DEMO_TIME
                    }
                    GameState::GameOver if self.effects.game_over_ready() => {
                        self.attract.wait += delta_time;
                        self.attract.wait >= GAME_OVER_TIME
                    }
                    _ => false,
                };
                if over {
                    self.end_demo();
                }
            }
            Phase::Scores => {
                self.attract.timer += delta_time;
                if self.attract.timer >= SCORES_TIME {
                    self.stop_attract();
                }
            }
        }
    }

    /// a key resets the idle time, or leaves the demo; true if the key was used
    pub(super) fn attract_key(&mut self) -> bool {
        self.attract.idle = 0.0;
        if self.attract.phase == Phase::Off {
            return false;
        }
        self.stop_attract();
        true
    }

    fn start_demo(&mut self) {
        self.new_game();
        self.level = DEMO_LEVEL;
        self.total_lines_cleared = (DEMO_LEVEL - 1) * LINES_PER_LEVEL;
        self.level_display = ScoreDisplay::new(DEMO_LEVEL);
        self.level_progress = LevelProgress::new(LINES_PER_LEVEL, 0);
        self.gravity_delay = gravity_delay(DEMO_LEVEL, self.mode);
        self.controls.clear_inputs();
        self.attract.reset(Phase::Demo);
        self.set_state(GameState::Playing);
    }

    fn end_demo(&mut self) {
        self.assets.end_game_over();
        self.background.thaw();
        self.effects = Effects::new();
        self.highlight = None;
        self.scores_from_game = false;
        self.attract.reset(Phase::Scores);
        self.set_state(GameState::HighScores);
    }

    /// back to the menu with a fresh board
    fn stop_attract(&mut self) {
        self.attract.reset(Phase::Off);
        self.new_game();
    }

    /// the computer's moves, quicker as gravity speeds up
    pub(super) fn computer(&mut self, delta_time: f64) {
        if self.playfield.is_clearing_lines() {
            return;
        }
        let Some(active) = self.playfield.active_rustomino.clone() else {
            self.attract.plan = None;
            return;
        };
        let y = active.translation.y;
        if self.attract.plan.is_none() || y > self.attract.piece_y {
            self.attract.plan = Some(best_plan(
                &self.playfield.slots,
                &active,
                self.next_rustomino.as_ref(),
                &mut self.attract.rng,
            ));
            self.attract.wait = THINK_TIME.min(self.gravity_delay * 2.0);
        }
        self.attract.piece_y = y;
        let step = MOVE_TIME.min(self.gravity_delay);
        self.attract.wait -= delta_time;
        while self.attract.wait <= 0.0 {
            self.attract.wait += step;
            let Some(plan) = self.attract.plan.as_mut() else {
                return;
            };
            let x = self
                .playfield
                .active_rustomino
                .as_ref()
                .map_or(plan.x, |r| r.translation.x);
            // blocked, drop where it is
            let moved = if plan.turns != 0 {
                let rotation = if plan.turns > 0 {
                    Rotation::Cw
                } else {
                    Rotation::Ccw
                };
                plan.turns -= plan.turns.signum();
                self.rotate(rotation)
            } else if x != plan.x {
                let direction = if plan.x < x {
                    TranslationDirection::Left
                } else {
                    TranslationDirection::Right
                };
                self.translate(direction)
            } else {
                false
            };
            if !moved {
                self.hard_drop();
                self.attract.plan = None;
                self.attract.piece_y = i32::MAX;
                return;
            }
        }
    }
}

/// the best placement for the piece, looking ahead to the next
/// the best placement, but now and then a worse one, to look human
fn best_plan(
    slots: &PlayfieldSlots,
    piece: &Rustomino,
    next: Option<&Rustomino>,
    rng: &mut Xoshiro256PlusPlus,
) -> Plan {
    let mut ranked: Vec<(f64, Plan)> = placements(slots, piece)
        .map(|(plan, dropped)| {
            let (board, first) = place(slots, &dropped);
            let score = match next {
                Some(next) => placements(&board, next)
                    .map(|(_, dropped)| {
                        let (board, second) = place(&board, &dropped);
                        first.moves + second.moves + evaluate(&board)
                    })
                    .fold(f64::NEG_INFINITY, f64::max),
                None => first.moves + evaluate(&board),
            };
            (score, plan)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    let pick = if rng.random_bool(SLIP_CHANCE) {
        rng.random_range(1..=SLIP_RANK)
            .min(ranked.len().saturating_sub(1))
    } else {
        0
    };
    ranked.get(pick).map_or(
        Plan {
            turns: 0,
            x: piece.translation.x,
        },
        |&(_, plan)| plan,
    )
}

/// every reachable turn and column, dropped
fn placements<'a>(
    slots: &'a PlayfieldSlots,
    piece: &Rustomino,
) -> impl Iterator<Item = (Plan, Rustomino)> + 'a {
    let (left, down) = (IVec2::NEG_X, IVec2::NEG_Y);
    let mut found = Vec::new();
    'turns: for turns in [0i32, 1, 2, -1] {
        let mut turned = piece.clone();
        let rotation = if turns < 0 {
            Rotation::Ccw
        } else {
            Rotation::Cw
        };
        for _ in 0..turns.abs() {
            let Some(kick) = check_rotation(slots, &turned, &rotation) else {
                continue 'turns;
            };
            turned.rotate(&rotation, &kick);
        }
        for dir in [left, -left] {
            let mut shifted = turned.clone();
            loop {
                // the unshifted column once
                if dir == left || shifted.translation.x != turned.translation.x {
                    let mut dropped = shifted.clone();
                    while !check_collision(slots, dropped.translated(&down)) {
                        dropped.translate(down);
                    }
                    let plan = Plan {
                        turns,
                        x: shifted.translation.x,
                    };
                    found.push((plan, dropped));
                }
                if check_collision(slots, shifted.translated(&dir)) {
                    break;
                }
                shifted.translate(dir);
            }
        }
    }
    found.into_iter()
}

/// what placing a piece scored, apart from the board it leaves
struct Placed {
    moves: f64,
    #[cfg_attr(not(test), allow(dead_code))]
    lines: usize,
}

/// lock the dropped piece and clear lines
fn place(slots: &PlayfieldSlots, piece: &Rustomino) -> (PlayfieldSlots, Placed) {
    let mut board = *slots;
    let blocks = piece.playfield_slots();
    for b in blocks {
        board[b.y as usize][b.x as usize] = SlotState::Locked(piece.rtype);
    }
    let (low, high) = blocks.iter().fold((i32::MAX, i32::MIN), |(lo, hi), b| {
        (lo.min(b.y), hi.max(b.y))
    });
    let full = complete_lines(&board);
    let eroded = full.len()
        * blocks
            .iter()
            .filter(|b| full.contains(&(b.y as usize)))
            .count();
    remove_lines(&mut board, &full);
    // over the visible rows tops out soon
    let above = if high >= PLAYFIELD_SIZE[1] {
        -1000.0
    } else {
        0.0
    };
    let landing = (low + high) as f64 / 2.0;
    let moves = -12.63 * landing + 6.60 * eroded as f64 + CLEAR_BONUS[full.len()] + above;
    let placed = Placed {
        moves,
        lines: full.len(),
    };
    (board, placed)
}

/// board features, weighted (Thiery and Scherrer's BCTS)
fn evaluate(board: &PlayfieldSlots) -> f64 {
    let filled =
        |x: i32, y: usize| x < 0 || x >= W as i32 || board[y][x as usize] != SlotState::Empty;
    let mut row_transitions = 0;
    for y in 0..H {
        let mut last = true;
        for x in 0..=W as i32 {
            let now = filled(x, y);
            row_transitions += (now != last) as u32;
            last = now;
        }
    }
    let (mut column_transitions, mut holes, mut hole_depth, mut wells) = (0, 0, 0, 0);
    let mut hole_rows = [false; H];
    let mut deepest = 0; // one column's wells, kept open for a big clear
    for x in 0..W as i32 {
        // up from the floor
        let mut last = true;
        for y in 0..H {
            let now = filled(x, y);
            column_transitions += (now != last) as u32;
            last = now;
        }
        let mut above = 0;
        let mut well = 0;
        let mut column_wells = 0;
        for y in (0..H).rev() {
            let now = filled(x, y);
            if now {
                above += 1;
            } else if above > 0 {
                holes += 1;
                hole_depth += above;
                hole_rows[y] = true;
            }
            if !now && filled(x - 1, y) && filled(x + 1, y) {
                well += 1;
                column_wells += well;
            } else {
                well = 0;
            }
        }
        wells += column_wells;
        deepest = deepest.max(column_wells);
    }
    // up to four deep
    wells -= deepest.min(WELL_FORGIVEN);
    let rows_with_holes = hole_rows.iter().filter(|&&h| h).count();
    -9.22 * row_transitions as f64
        - 19.77 * column_transitions as f64
        - 13.08 * holes as f64
        - 10.49 * wells as f64
        - 1.61 * hole_depth as f64
        - 24.04 * rows_with_holes as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rustomino::RustominoBag;

    /// plays a long game without topping out
    #[test]
    fn computer_plays_well() {
        let mut slots = [[SlotState::Empty; W]; H];
        let mut bag = RustominoBag::new();
        let (mut piece, mut next) = (bag.get_next(), bag.get_next());
        let mut rng = rand::make_rng();
        let mut lines = 0;
        let mut clears = [0; 5];
        for _ in 0..1000 {
            assert!(
                !check_collision(&slots, piece.playfield_slots()),
                "topped out at {lines} lines"
            );
            let plan = best_plan(&slots, &piece, Some(&next), &mut rng);
            let (_, dropped) = placements(&slots, &piece)
                .find(|(p, _)| *p == plan)
                .expect("the plan is a placement");
            let (board, placed) = place(&slots, &dropped);
            slots = board;
            lines += placed.lines;
            clears[placed.lines] += 1;
            piece = std::mem::replace(&mut next, bag.get_next());
        }
        assert!(lines >= 390, "{lines} lines");
        // more multi-line clears than singles
        let multi: usize = clears[2..].iter().sum();
        assert!(multi > clears[1], "clears {:?}", &clears[1..]);
    }
}
