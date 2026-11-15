use ggez::glam::{IVec2, Vec2};
use ggez::graphics::Color;
use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::playfield::{Playfield, SlotState, PLAYFIELD_SLOTS};

const GRAVITY: f32 = 30.0; // cells per second squared
const LINE_CLEAR_GOLD: Color = Color::new(1.0, 0.92, 0.72, 1.0);
// per lines cleared: particles per block, speed, color mix to gold, flash, shake px, shake secs
const TIERS: [(usize, f32, f32, f32, f32, f32); 4] = [
    (2, 1.0, 0.7, 0.0, 0.0, 0.0),
    (3, 1.2, 0.45, 0.1, 0.0, 0.0),
    (4, 1.4, 0.2, 0.2, 3.0, 0.25),
    (7, 1.8, 0.0, 0.45, 8.0, 0.45),
];
const FLASH_TIME: f32 = 0.3;
const CALLOUT_TIME: f32 = 1.0;
const CALLOUT_POP_TIME: f32 = 0.15;
const TRAIL_TIME: f32 = 0.18;
const IMPACT_TIME: f32 = 0.15;
const BUMP_TIME: f32 = 0.2;
const BUMP_MIN: f32 = 1.5; // px
const BUMP_PER_ROW: f32 = 0.15;
const BUMP_MAX: f32 = 4.0;
const DUST_PER_COLUMN: usize = 3;
const SOFT_TRAIL: f32 = 0.5; // of a hard drop trail
const GRAVITY_TRAIL: f32 = 0.5; // of a hard drop trail
const LOCK_FLASH: f32 = 0.5; // of a hard drop flash
const LOCK_BUMP: f32 = 1.0; // px
const LOCK_DUST_PER_COLUMN: usize = 1;
const TOPOUT_SHAKE: f32 = 6.0; // px
const TOPOUT_SHAKE_TIME: f32 = 0.35;
const CURTAIN_ROW_DELAY: f32 = 0.035; // bottom row first
const CURTAIN_ROW_TIME: f32 = 0.15;
const CURTAIN_END: f32 = (PLAYFIELD_SLOTS[1] - 1) as f32 * CURTAIN_ROW_DELAY + CURTAIN_ROW_TIME;
const OVERLAY_FADE_TIME: f32 = 0.3;
const PANEL_DROP_TIME: f32 = 0.45;
const LETTER_START: f32 = 0.2; // after the overlay starts
const LETTER_DELAY: f32 = 0.06;
const LETTER_DROP_TIME: f32 = 0.4;
const COUNT_START: f32 = 0.7; // after the overlay starts
const COUNT_TIME: f32 = 0.8;
const READY_TIME: f32 = 0.4; // after the overlay starts

pub struct Trail {
    pub x: f32, // playfield cells, y up
    pub from: f32,
    pub to: f32,
    pub color: Color,
    pub strength: f32,
    time: f32,
}

impl Trail {
    /// 1.0 at spawn, falling to 0.0
    pub fn fade(&self) -> f32 {
        (1.0 - self.time / TRAIL_TIME).max(0.0)
    }
}

pub struct Impact {
    pub blocks: [IVec2; 4],
    pub strength: f32,
    time: f32,
}

impl Impact {
    /// 1.0 at landing, falling to 0.0
    pub fn fade(&self) -> f32 {
        (1.0 - self.time / IMPACT_TIME).max(0.0)
    }
}

/// board graying out row by row, then the overlay dropping in
pub struct GameOver {
    time: f32,
}

impl GameOver {
    pub fn time(&self) -> f32 {
        self.time
    }

    /// 0.0 colored to 1.0 gray
    pub fn gray(&self, y: i32) -> f32 {
        ((self.time - y as f32 * CURTAIN_ROW_DELAY) / CURTAIN_ROW_TIME).clamp(0.0, 1.0)
    }

    /// flash as a row turns gray
    pub fn flash(&self, y: i32) -> f32 {
        let g = self.gray(y);
        4.0 * g * (1.0 - g)
    }

    fn overlay_time(&self) -> f32 {
        (self.time - CURTAIN_END).max(0.0)
    }

    /// overlay dim, 0.0 to 1.0
    pub fn overlay(&self) -> f32 {
        (self.overlay_time() / OVERLAY_FADE_TIME).min(1.0)
    }

    /// panel drop, 0.0 to 1.0
    pub fn panel(&self) -> f32 {
        (self.overlay_time() / PANEL_DROP_TIME).min(1.0)
    }

    /// drop of title letter i, 0.0 to 1.0
    pub fn letter(&self, i: usize) -> f32 {
        let start = LETTER_START + i as f32 * LETTER_DELAY;
        ((self.overlay_time() - start) / LETTER_DROP_TIME).clamp(0.0, 1.0)
    }

    /// final score count up, 0.0 to 1.0
    pub fn count(&self) -> f32 {
        ((self.overlay_time() - COUNT_START) / COUNT_TIME).clamp(0.0, 1.0)
    }

    /// accepts a key to continue
    pub fn ready(&self) -> bool {
        self.overlay_time() >= READY_TIME
    }
}

pub struct Callout {
    pub lines: usize,
    pub row: f32, // playfield cells, y up
    time: f32,
}

impl Callout {
    pub fn text(&self) -> &'static str {
        match self.lines {
            2 => "Double",
            3 => "Triple",
            _ => "Quad!",
        }
    }

    /// fraction of the callout's life, 0.0 to 1.0
    pub fn progress(&self) -> f32 {
        (self.time / CALLOUT_TIME).min(1.0)
    }

    /// 1.0 when it appears, falling to 0.0
    pub fn pop(&self) -> f32 {
        (1.0 - self.time / CALLOUT_POP_TIME).max(0.0)
    }
}

pub struct Particle {
    pub pos: Vec2, // playfield cells, y up
    vel: Vec2,
    pub color: Color,
    pub size: f32, // fraction of a block
    delay: f32,
    age: f32,
    life: f32,
}

impl Particle {
    pub fn visible(&self) -> bool {
        self.delay <= 0.0
    }

    /// 1.0 at spawn, falling to 0.0
    pub fn fade(&self) -> f32 {
        (1.0 - self.age / self.life).clamp(0.0, 1.0)
    }
}

pub struct Effects {
    pub particles: Vec<Particle>,
    pub callout: Option<Callout>,
    pub trails: Vec<Trail>,
    pub impact: Option<Impact>,
    pub game_over: Option<GameOver>,
    flash: f32,
    flash_time: f32,
    shake: f32,
    shake_time: f32,
    shake_duration: f32,
    bump: f32,
    bump_time: f32,
    rng: Xoshiro256PlusPlus,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            callout: None,
            trails: Vec::new(),
            impact: None,
            game_over: None,
            flash: 0.0,
            flash_time: FLASH_TIME,
            shake: 0.0,
            shake_time: 0.0,
            shake_duration: 0.0,
            bump: 0.0,
            bump_time: BUMP_TIME,
            rng: Xoshiro256PlusPlus::seed_from_u64(0x5eed),
        }
    }

    /// burst the blocks of the lines being cleared
    pub fn line_clear(&mut self, playfield: &Playfield) {
        let Some(line_clear) = &playfield.line_clear else {
            return;
        };
        let tier = line_clear.lines.len().clamp(1, TIERS.len()) - 1;
        let (count, speed, gold, flash, shake, shake_duration) = TIERS[tier];
        let duration = line_clear.duration as f32;
        for &y in &line_clear.lines {
            for (x, slot) in playfield.slots[y].iter().enumerate() {
                let SlotState::Locked(rtype) = slot else {
                    continue;
                };
                let delay = line_clear.wipe_start(x) as f32 * duration + 0.05;
                let away = (x as f32 - line_clear.origin(x) as f32).signum();
                for _ in 0..count {
                    let color = if self.rng.random_bool(0.2) {
                        Color::WHITE
                    } else {
                        mix(rtype.color(), LINE_CLEAR_GOLD, gold)
                    };
                    let vel = Vec2::new(
                        away * self.rng.random_range(0.0..6.0) + self.rng.random_range(-3.0..3.0),
                        self.rng.random_range(4.0..12.0),
                    ) * speed;
                    self.particles.push(Particle {
                        pos: Vec2::new(
                            x as f32 + self.rng.random_range(0.2..0.8),
                            y as f32 + self.rng.random_range(0.2..0.8),
                        ),
                        vel,
                        color,
                        size: self.rng.random_range(0.15..0.35),
                        delay,
                        age: 0.0,
                        life: self.rng.random_range(0.5..1.0),
                    });
                }
            }
        }
        let lines = line_clear.lines.len();
        self.callout = (lines >= 2).then(|| Callout {
            lines,
            row: line_clear
                .lines
                .iter()
                .max()
                .map_or(0.0, |&top| top as f32 + 1.5),
            time: 0.0,
        });
        if flash > 0.0 {
            self.flash = flash;
            self.flash_time = 0.0;
        }
        if shake > 0.0 {
            self.shake = shake;
            self.shake_time = 0.0;
            self.shake_duration = shake_duration;
        }
    }

    /// trail, landing flash, dust and bump for a hard drop
    pub fn hard_drop(&mut self, from: &[IVec2; 4], to: &[IVec2; 4], color: Color) {
        let rows = (from[0].y - to[0].y) as f32;
        self.drop_trails(from, to, color, 1.0);
        for x in columns(to) {
            let bottom = to
                .iter()
                .filter(|b| b.x == x)
                .map(|b| b.y)
                .min()
                .unwrap_or(0);
            self.dust(x, bottom, color, DUST_PER_COLUMN);
        }
        self.impact = Some(Impact {
            blocks: *to,
            strength: 1.0,
            time: 0.0,
        });
        self.bump = (BUMP_MIN + rows * BUMP_PER_ROW).min(BUMP_MAX);
        self.bump_time = 0.0;
    }

    /// streak over rows skipped by one gravity tick
    pub fn gravity_drop(&mut self, from: &[IVec2; 4], to: &[IVec2; 4], color: Color) {
        self.drop_trails(from, to, color, GRAVITY_TRAIL);
    }

    /// one trail per column from the old top to the new top
    fn drop_trails(&mut self, from: &[IVec2; 4], to: &[IVec2; 4], color: Color, strength: f32) {
        if from[0].y <= to[0].y {
            return;
        }
        for x in columns(to) {
            let top = |blocks: &[IVec2; 4]| blocks.iter().filter(|b| b.x == x).map(|b| b.y).max();
            if let (Some(from_top), Some(to_top)) = (top(from), top(to)) {
                self.trails.push(Trail {
                    x: x as f32,
                    from: from_top as f32 + 1.0,
                    to: to_top as f32 + 1.0,
                    color,
                    strength,
                    time: 0.0,
                });
            }
        }
    }

    /// short streak above a piece moved down one row
    pub fn soft_drop(&mut self, to: &[IVec2; 4], color: Color) {
        for x in columns(to) {
            let Some(top) = to.iter().filter(|b| b.x == x).map(|b| b.y).max() else {
                continue;
            };
            self.trails.push(Trail {
                x: x as f32,
                from: top as f32 + 2.0,
                to: top as f32 + 1.0,
                color,
                strength: SOFT_TRAIL,
                time: 0.0,
            });
        }
    }

    /// soft flash, dust and bump for a normal lock
    pub fn lock(&mut self, blocks: &[IVec2; 4], color: Color) {
        for x in columns(blocks) {
            if let Some(bottom) = blocks.iter().filter(|b| b.x == x).map(|b| b.y).min() {
                self.dust(x, bottom, color, LOCK_DUST_PER_COLUMN);
            }
        }
        self.impact = Some(Impact {
            blocks: *blocks,
            strength: LOCK_FLASH,
            time: 0.0,
        });
        self.bump = LOCK_BUMP;
        self.bump_time = 0.0;
    }

    fn dust(&mut self, x: i32, bottom: i32, color: Color, count: usize) {
        for _ in 0..count {
            let side = if self.rng.random_bool(0.5) { 1.0 } else { -1.0 };
            self.particles.push(Particle {
                pos: Vec2::new(x as f32 + self.rng.random_range(0.1..0.9), bottom as f32),
                vel: Vec2::new(
                    side * self.rng.random_range(2.0..6.0),
                    self.rng.random_range(1.0..4.0),
                ),
                color: mix(color, Color::WHITE, 0.6),
                size: self.rng.random_range(0.08..0.18),
                delay: 0.0,
                age: 0.0,
                life: self.rng.random_range(0.2..0.4),
            });
        }
    }

    /// top out shake and curtain
    pub fn game_over(&mut self) {
        self.game_over = Some(GameOver { time: 0.0 });
        self.shake = TOPOUT_SHAKE;
        self.shake_time = 0.0;
        self.shake_duration = TOPOUT_SHAKE_TIME;
    }

    pub fn game_over_ready(&self) -> bool {
        self.game_over.as_ref().is_none_or(GameOver::ready)
    }

    pub fn update(&mut self, delta_time: f64) {
        let dt = delta_time as f32;
        if let Some(game_over) = &mut self.game_over {
            game_over.time += dt;
        }
        self.flash_time += dt;
        self.shake_time += dt;
        self.bump_time += dt;
        for trail in &mut self.trails {
            trail.time += dt;
        }
        self.trails.retain(|trail| trail.time < TRAIL_TIME);
        if let Some(impact) = &mut self.impact {
            impact.time += dt;
            if impact.time >= IMPACT_TIME {
                self.impact = None;
            }
        }
        for particle in &mut self.particles {
            if particle.delay > 0.0 {
                particle.delay -= dt;
                continue;
            }
            particle.age += dt;
            particle.vel.y -= GRAVITY * dt;
            particle.pos += particle.vel * dt;
        }
        self.particles
            .retain(|particle| particle.age < particle.life);
        if let Some(callout) = &mut self.callout {
            callout.time += dt;
            if callout.time >= CALLOUT_TIME {
                self.callout = None;
            }
        }
    }

    /// playfield flash alpha
    pub fn flash(&self) -> f32 {
        self.flash * (1.0 - self.flash_time / FLASH_TIME).max(0.0)
    }

    /// screen offset in pixels
    pub fn shake(&self) -> Vec2 {
        self.jitter() + self.bump()
    }

    fn jitter(&self) -> Vec2 {
        if self.shake_time >= self.shake_duration {
            return Vec2::ZERO;
        }
        let decay = 1.0 - self.shake_time / self.shake_duration;
        let t = self.shake_time;
        Vec2::new((t * 90.0).sin(), (t * 77.0).cos()) * self.shake * decay
    }

    /// downward knock that springs back
    fn bump(&self) -> Vec2 {
        if self.bump_time >= BUMP_TIME {
            return Vec2::ZERO;
        }
        let decay = 1.0 - self.bump_time / BUMP_TIME;
        Vec2::new(
            0.0,
            (self.bump_time * 40.0).cos() * self.bump * decay * decay,
        )
    }
}

fn columns(blocks: &[IVec2; 4]) -> Vec<i32> {
    let mut columns: Vec<i32> = blocks.iter().map(|block| block.x).collect();
    columns.sort_unstable();
    columns.dedup();
    columns
}

fn mix(from: Color, to: Color, amount: f32) -> Color {
    let lerp = |a: f32, b: f32| a + (b - a) * amount;
    Color::new(
        lerp(from.r, to.r),
        lerp(from.g, to.g),
        lerp(from.b, to.b),
        1.0,
    )
}
