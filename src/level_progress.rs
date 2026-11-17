use crate::gfx::Vec2;
use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

const FILL_DELAY: f32 = 0.05; // after the clear starts
const FILL_STAGGER: f32 = 0.07; // between segments
const GROW_TIME: f32 = 0.2;
const COOL_TIME: f32 = 0.5; // white hot to gold
const POP_HEIGHT: f32 = 1.4; // extra height as a segment appears
const SPARKS: usize = 7; // per filled segment
const BURST_SPARKS: usize = 12; // per segment on level up
const BURST_SPEED: f32 = 1.7;
const SPARK_GRAVITY: f32 = 520.0; // px/s²
const SWEEP_TIME: f32 = 0.28;
const SWEEP_WIDTH: f32 = 1.6; // segments
const SWEEP_DRIFT: f32 = 4.0; // segments/s, sparks lean with the sweep
const BURST_TIME: f32 = 0.45;
const DRAIN_START: f32 = SWEEP_TIME + 0.3;
const DRAIN_STAGGER: f32 = 0.03;
const DRAIN_TIME: f32 = 0.14;
const PULSE_SPEED: f32 = 7.0; // rad/s

pub struct Spark {
    pub pos: Vec2, // x in segments from the left, y in px from the bar center
    vel: Vec2,
    pub size: f32, // px
    pub white: bool,
    age: f32,
    life: f32,
}

impl Spark {
    /// 1.0 at spawn, falling to 0.0
    pub fn fade(&self) -> f32 {
        (1.0 - self.age / self.life).clamp(0.0, 1.0)
    }
}

/// how a filled segment is drawn, sizes relative to its rest size
pub struct SegmentLook {
    pub width: f32,
    pub height: f32,
    pub heat: f32, // 1.0 white, 0.0 accent
    pub glow: f32,
}

/// segments filling toward the next level, with a sweep and burst on level up
pub struct LevelProgress {
    fills: Vec<Option<f32>>, // seconds since filled, negative while waiting
    level_up: Option<f32>,   // seconds since the sweep started, negative while waiting
    carry: usize,            // lines past the level up
    time: f32,
    pub sparks: Vec<Spark>,
    rng: Xoshiro256PlusPlus,
}

impl LevelProgress {
    pub fn new(per_level: usize, filled: usize) -> Self {
        let mut fills = vec![None; per_level];
        for fill in fills.iter_mut().take(filled) {
            *fill = Some(COOL_TIME);
        }
        Self {
            fills,
            level_up: None,
            carry: 0,
            time: 0.0,
            sparks: Vec::new(),
            rng: Xoshiro256PlusPlus::seed_from_u64(0x1e7e1),
        }
    }

    pub fn len(&self) -> usize {
        self.fills.len()
    }

    pub fn add(&mut self, lines: usize) {
        if self.level_up.is_some() {
            self.carry += lines;
            return;
        }
        // after any segments still waiting
        let mut delay = self
            .fills
            .iter()
            .flatten()
            .filter(|t| **t < 0.0)
            .map(|t| -t + FILL_STAGGER)
            .fold(FILL_DELAY, f32::max);
        let mut lines = lines;
        for fill in self.fills.iter_mut().filter(|fill| fill.is_none()) {
            if lines == 0 {
                break;
            }
            *fill = Some(-delay);
            delay += FILL_STAGGER;
            lines -= 1;
        }
        if self.fills.iter().all(Option::is_some) {
            let last = self.fills.iter().flatten().map(|t| -t).fold(0.0, f32::max);
            self.level_up = Some(-(last + GROW_TIME));
            self.carry += lines;
        }
    }

    /// returns true on the frame the level up bursts
    pub fn update(&mut self, delta_time: f64) -> bool {
        let dt = delta_time as f32;
        self.time += dt;
        for i in 0..self.fills.len() {
            let Some(t) = self.fills[i] else {
                continue;
            };
            if t < 0.0 && t + dt >= 0.0 {
                self.spark(i, SPARKS, 1.0, 0.0);
            }
            self.fills[i] = Some(t + dt);
        }

        let mut burst = false;
        if let Some(t) = self.level_up {
            let next = t + dt;
            // sparks as the sweep passes each segment
            let (from, to) = (self.sweep_pos(t), self.sweep_pos(next));
            for i in 0..self.fills.len() {
                let mid = i as f32 + 0.5;
                if from < mid && mid <= to {
                    self.spark(i, BURST_SPARKS, BURST_SPEED, SWEEP_DRIFT);
                }
            }
            if t < SWEEP_TIME && next >= SWEEP_TIME {
                burst = true;
            }
            self.level_up = Some(next);
            if next >= self.drain_end() {
                self.fills.fill(None);
                self.level_up = None;
                let carry = std::mem::take(&mut self.carry);
                if carry > 0 {
                    self.add(carry);
                }
            }
        }

        for spark in &mut self.sparks {
            spark.age += dt;
            spark.vel.y += SPARK_GRAVITY * dt;
            spark.pos += spark.vel * dt;
        }
        self.sparks.retain(|spark| spark.age < spark.life);
        burst
    }

    fn spark(&mut self, segment: usize, count: usize, speed: f32, drift: f32) {
        for _ in 0..count {
            self.sparks.push(Spark {
                pos: Vec2::new(
                    segment as f32 + self.rng.random_range(0.2..0.8),
                    self.rng.random_range(-2.0..2.0),
                ),
                vel: Vec2::new(
                    self.rng.random_range(-2.5..2.5),
                    -self.rng.random_range(70.0..200.0),
                ) * speed
                    + Vec2::new(drift, 0.0),
                size: self.rng.random_range(1.5..3.5),
                white: self.rng.random_bool(0.4),
                age: 0.0,
                life: self.rng.random_range(0.3..0.7),
            });
        }
    }

    fn drain_end(&self) -> f32 {
        DRAIN_START + self.fills.len().saturating_sub(1) as f32 * DRAIN_STAGGER + DRAIN_TIME
    }

    /// 1.0 full size, shrinking to 0.0 after the burst
    fn drain(&self, i: usize) -> f32 {
        let Some(t) = self.level_up else {
            return 1.0;
        };
        let start = DRAIN_START + i as f32 * DRAIN_STAGGER;
        1.0 - ((t - start) / DRAIN_TIME).clamp(0.0, 1.0)
    }

    /// sweep center in segments, from before the bar to past its end
    fn sweep_pos(&self, t: f32) -> f32 {
        t.clamp(0.0, SWEEP_TIME) / SWEEP_TIME * (self.len() as f32 + 2.0) - 1.0
    }

    /// bright band crossing the full bar
    fn sweep(&self, i: usize) -> f32 {
        match self.level_up {
            Some(t) if (0.0..SWEEP_TIME).contains(&t) => {
                (1.0 - (self.sweep_pos(t) - (i as f32 + 0.5)).abs() / SWEEP_WIDTH).max(0.0)
            }
            _ => 0.0,
        }
    }

    /// 1.0 at the level up burst, falling to 0.0
    pub fn burst(&self) -> f32 {
        match self.level_up {
            Some(t) if t >= SWEEP_TIME => (1.0 - (t - SWEEP_TIME) / BURST_TIME).max(0.0),
            _ => 0.0,
        }
    }

    pub fn segment(&self, i: usize) -> Option<SegmentLook> {
        let t = self.fills[i].filter(|t| *t >= 0.0)?;
        let drain = self.drain(i);
        if drain <= 0.0 {
            return None;
        }
        let grow = ease_out_back((t / GROW_TIME).min(1.0));
        let settle = (1.0 - t / (2.0 * GROW_TIME)).max(0.0);
        let cool = (1.0 - t / COOL_TIME).max(0.0);
        let flare = self.sweep(i).max(self.burst());
        Some(SegmentLook {
            width: grow * drain,
            height: (1.0 + POP_HEIGHT * settle * settle) * drain,
            heat: (cool * cool).max(flare),
            glow: cool.max(flare),
        })
    }

    /// the last empty segment pulsing when one line from leveling up
    pub fn pulse(&self) -> Option<(usize, f32)> {
        if self.level_up.is_some() {
            return None;
        }
        let mut empty = (0..self.len()).filter(|&i| self.fills[i].is_none());
        let i = empty.next()?;
        if empty.next().is_some() {
            return None;
        }
        Some((i, 0.5 - 0.5 * (self.time * PULSE_SPEED).cos()))
    }
}

/// overshoots past 1.0 then settles
fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}
