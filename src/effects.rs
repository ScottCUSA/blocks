use ggez::glam::Vec2;
use ggez::graphics::Color;
use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::playfield::{Playfield, SlotState};

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
    flash: f32,
    flash_time: f32,
    shake: f32,
    shake_time: f32,
    shake_duration: f32,
    rng: Xoshiro256PlusPlus,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            flash: 0.0,
            flash_time: FLASH_TIME,
            shake: 0.0,
            shake_time: 0.0,
            shake_duration: 0.0,
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

    pub fn update(&mut self, delta_time: f64) {
        let dt = delta_time as f32;
        self.flash_time += dt;
        self.shake_time += dt;
        for particle in &mut self.particles {
            if particle.delay > 0.0 {
                particle.delay -= dt;
                continue;
            }
            particle.age += dt;
            particle.vel.y -= GRAVITY * dt;
            particle.pos += particle.vel * dt;
        }
        self.particles.retain(|particle| particle.age < particle.life);
    }

    /// playfield flash alpha
    pub fn flash(&self) -> f32 {
        self.flash * (1.0 - self.flash_time / FLASH_TIME).max(0.0)
    }

    /// screen offset in pixels
    pub fn shake(&self) -> Vec2 {
        if self.shake_time >= self.shake_duration {
            return Vec2::ZERO;
        }
        let decay = 1.0 - self.shake_time / self.shake_duration;
        let t = self.shake_time;
        Vec2::new((t * 90.0).sin(), (t * 77.0).cos()) * self.shake * decay
    }
}

fn mix(from: Color, to: Color, amount: f32) -> Color {
    let lerp = |a: f32, b: f32| a + (b - a) * amount;
    Color::new(lerp(from.r, to.r), lerp(from.g, to.g), lerp(from.b, to.b), 1.0)
}
