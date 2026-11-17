//! attract mode: idle at the menu, the computer plays a demo game, then the high scores

use bevy::app::RunFixedMainLoop;
use bevy::prelude::*;
use rand::RngExt;
use rand_xoshiro::Xoshiro256PlusPlus;

use super::board::{gravity_delay, Board, Mode, Play, Score};
use super::input::{collect_keys, HandleKeys, Keys};
use super::{ScoreView, Screen, Sim, Viewer, LINES_PER_LEVEL};
use crate::background::Background;
use crate::effects::{reset_effects, BoardFx};
use crate::gfx::IVec2;
use crate::level_progress::LevelProgress;
use crate::playfield::TranslationDirection;
use crate::playfield::{PlayfieldSlots, SlotState, PLAYFIELD_SIZE, PLAYFIELD_SLOTS};
use crate::rustomino::{Rotation, Rustomino};
use crate::score_display::ScoreDisplay;

const IDLE_TIME: f64 = 30.0; // at the menu before a demo
const DEMO_TIME: f64 = 30.0; // longest demo game
const GAME_OVER_TIME: f64 = 3.0; // game over shown, once the overlay is in
const SCORES_TIME: f64 = 10.0; // high scores shown after a demo
const MOVE_TIME: f64 = 0.11; // between moves, at slow gravity
const THINK_TIME: f64 = 0.25; // before the first move
const DEMO_LEVEL: usize = 7;
/// by lines cleared, toward bigger clears
const CLEAR_BONUS: [f64; 5] = [0.0, -60.0, 0.0, 150.0, 400.0];
const WELL_FORGIVEN: u32 = 10; // 1 + 2 + 3 + 4
const SLIP_CHANCE: f64 = 0.1; // of a worse placement
const SLIP_RANK: usize = 3; // worst placement ranked from the best

const W: usize = PLAYFIELD_SLOTS[0];
const H: usize = PLAYFIELD_SLOTS[1];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
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

#[derive(Resource, Debug, Default)]
pub struct Attract {
    pub phase: Phase,
    idle: f64,
    timer: f64,
    plan: Option<Plan>,
    piece_y: i32, // the planned piece's row, a higher one is a new piece
    wait: f64,
}

impl Attract {
    pub fn demo(&self) -> bool {
        self.phase == Phase::Demo
    }
}

/// the player has the game
pub fn player(attract: Res<Attract>) -> bool {
    attract.phase == Phase::Off
}

pub struct AttractPlugin;

impl Plugin for AttractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Attract>()
            .add_systems(OnEnter(Screen::Menu), stop)
            .add_systems(
                RunFixedMainLoop,
                attract
                    .after(collect_keys)
                    .in_set(HandleKeys)
                    .run_if(in_state(Viewer::Off)),
            )
            .add_systems(
                FixedUpdate,
                computer
                    .before(Sim::Play)
                    .run_if(in_state(Screen::Playing))
                    .run_if(in_state(Viewer::Off)),
            );
    }
}

fn stop(mut attract: ResMut<Attract>) {
    *attract = Attract::default();
}

/// start a demo when idle, leave it on any key
#[allow(clippy::too_many_arguments)]
fn attract(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<Keys>,
    screen: Res<State<Screen>>,
    fx: Res<BoardFx>,
    mut attract: ResMut<Attract>,
    mode: Res<Mode>,
    mut board: ResMut<Board>,
    mut score: ResMut<Score>,
    mut background: ResMut<Background>,
    mut score_view: ResMut<ScoreView>,
    mut next: ResMut<NextState<Screen>>,
) {
    let dt = time.delta_secs_f64();
    let pressed = !keys.pressed.is_empty();
    let screen = *screen.get();
    match attract.phase {
        Phase::Off => {
            attract.idle = if pressed || screen != Screen::Menu {
                0.0
            } else {
                attract.idle + dt
            };
            if attract.idle >= IDLE_TIME {
                *attract = Attract {
                    phase: Phase::Demo,
                    ..default()
                };
                start_level(&mut board, &mut score, *mode);
                next.set(Screen::Playing);
            }
        }
        _ if pressed => next.set(Screen::Menu),
        Phase::Demo => {
            attract.timer += dt;
            let over = match screen {
                Screen::GameOver if fx.game_over_ready() => {
                    attract.wait += dt;
                    attract.wait >= GAME_OVER_TIME
                }
                Screen::Playing => attract.timer >= DEMO_TIME,
                _ => false,
            };
            if over {
                background.thaw();
                commands.run_system_cached(reset_effects);
                score_view.highlight = None;
                score_view.from_game = false;
                attract.phase = Phase::Scores;
                attract.timer = 0.0;
                next.set(Screen::HighScores);
            }
        }
        Phase::Scores => {
            attract.timer += dt;
            if attract.timer >= SCORES_TIME {
                next.set(Screen::Menu);
            }
        }
    }
}

/// the demo starts further in
fn start_level(board: &mut Board, score: &mut Score, mode: Mode) {
    score.level = DEMO_LEVEL;
    score.lines = (DEMO_LEVEL - 1) * LINES_PER_LEVEL;
    score.level_display = ScoreDisplay::new(DEMO_LEVEL);
    score.level_progress = LevelProgress::new(LINES_PER_LEVEL, 0);
    board.gravity_delay = gravity_delay(DEMO_LEVEL, mode);
}

/// the computer's moves, quicker as gravity speeds up
fn computer(
    time: Res<Time>,
    mut attract: ResMut<Attract>,
    mut play: Play,
    mut rng: Local<Option<Xoshiro256PlusPlus>>,
) {
    if !attract.demo() || play.board.topped_out {
        return;
    }
    let Some(active) = play.board.playfield.active_rustomino.clone() else {
        attract.plan = None;
        return;
    };
    let y = active.translation.y;
    if attract.plan.is_none() || y > attract.piece_y {
        let next = play.board.next.clone();
        attract.plan = Some(best_plan(
            &play.board.playfield.slots,
            &active,
            next.as_ref(),
            rng.get_or_insert_with(rand::make_rng),
        ));
        attract.wait = THINK_TIME.min(play.board.gravity_delay * 2.0);
    }
    attract.piece_y = y;
    let step = MOVE_TIME.min(play.board.gravity_delay);
    attract.wait -= time.delta_secs_f64();
    while attract.wait <= 0.0 {
        attract.wait += step;
        let Some(plan) = attract.plan.as_mut() else {
            return;
        };
        let x = play
            .board
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
            play.rotate(rotation)
        } else if x != plan.x {
            let direction = if plan.x < x {
                TranslationDirection::Left
            } else {
                TranslationDirection::Right
            };
            play.translate(direction)
        } else {
            false
        };
        if !moved {
            play.hard_drop();
            attract.plan = None;
            attract.piece_y = i32::MAX;
            return;
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

fn collides(slots: &PlayfieldSlots, blocks: [IVec2; 4]) -> bool {
    blocks.iter().any(|b| {
        b.x < 0
            || b.x >= W as i32
            || b.y < 0
            || b.y >= H as i32
            || slots[b.y as usize][b.x as usize] != SlotState::Empty
    })
}

/// every reachable turn and column, dropped
fn placements<'a>(
    slots: &'a PlayfieldSlots,
    piece: &Rustomino,
) -> impl Iterator<Item = (Plan, Rustomino)> + 'a {
    let left = IVec2::new(-1, 0);
    let down = IVec2::new(0, -1);
    let mut found = Vec::new();
    'turns: for turns in [0i32, 1, 2, -1] {
        let mut turned = piece.clone();
        let rotation = if turns < 0 {
            Rotation::Ccw
        } else {
            Rotation::Cw
        };
        for _ in 0..turns.abs() {
            let rotated = turned.rotated(&rotation);
            let Some(kick) = turned
                .wall_kick_tests(&rotation)
                .into_iter()
                .find(|kick| !collides(slots, crate::rustomino::translated(&rotated, kick)))
            else {
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
                    while !collides(slots, dropped.translated(&down)) {
                        dropped.translate(down);
                    }
                    let plan = Plan {
                        turns,
                        x: shifted.translation.x,
                    };
                    found.push((plan, dropped));
                }
                if collides(slots, shifted.translated(&dir)) {
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
    let full: Vec<usize> = (0..H)
        .filter(|&y| board[y].iter().all(|s| *s != SlotState::Empty))
        .collect();
    let eroded = full.len()
        * blocks
            .iter()
            .filter(|b| full.contains(&(b.y as usize)))
            .count();
    let mut write = 0;
    for read in 0..H {
        if !full.contains(&read) {
            board[write] = board[read];
            write += 1;
        }
    }
    for row in &mut board[write..] {
        *row = [SlotState::Empty; W];
    }
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
                !collides(&slots, piece.playfield_slots()),
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
