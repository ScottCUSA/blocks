#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundStyle {
    Stars,
    Plasma,
    Storm,
    Off,
}

impl BackgroundStyle {
    pub const ALL: [BackgroundStyle; 4] = [
        BackgroundStyle::Stars,
        BackgroundStyle::Plasma,
        BackgroundStyle::Storm,
        BackgroundStyle::Off,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BackgroundStyle::Stars => "Stars",
            BackgroundStyle::Plasma => "Plasma",
            BackgroundStyle::Storm => "Storm",
            BackgroundStyle::Off => "Off",
        }
    }
}

const LEVEL_SPEEDUP: f32 = 0.06;
const MAX_LEVEL_SPEED: f32 = 2.0;
const SURGE_SPEED: f32 = 2.5; // extra speed at full energy
const SURGE_DECAY: f32 = 1.5; // energy lost per second
const SURGE: [f32; 4] = [0.25, 0.45, 0.7, 1.0]; // energy per lines cleared
pub const STAR_FIELDS: usize = 64;
pub const LEVEL_UP_TIME: f32 = 1.3; // seconds
const RUSH_SPEED: f32 = 1.5; // extra speed at the rush peak
pub const FREEZE_TIME: f32 = 2.5; // seconds to slow to a stop
pub const THAW_TIME: f32 = 2.0; // seconds to thaw
const SKIP_STEP: f32 = 1.0 / 60.0;
const STORM_LEVELS: f32 = 11.0; // levels from calm to full storm, full at level 12
const WHITEOUT_START: f32 = 10.0; // level the whiteout starts to creep in
const WHITEOUT_LEVELS: f32 = 6.0; // levels from the first haze to a near whiteout
const STORM_RUSH_SEA: f32 = 2.0; // extra wave speed at the height of a level up
const SEA_LEVEL_UP_RAMP: f32 = 4.0; // how quickly the sea follows the level during a level up, per second
const STORM_RAMP: f32 = 0.4; // how quickly the storm follows the level, per second

/// animated background shown behind the game
pub struct Background {
    pub style: BackgroundStyle,
    pub time: f32,         // animation clock, runs faster with level and energy
    pub energy: f32,       // 0.0 to 1.0, raised by line clears
    level_up: Option<f32>, // seconds since a level up
    pub stars: usize,      // star field, replaced on level up
    freeze: Option<f32>,   // seconds since game over
    freeze_rate: f32,      // speed at game over
    frozen_at: f32,        // time at game over
    thaw: Option<f32>,     // seconds since the thaw started
    pub twinkle: f32,      // star twinkle clock, runs while frozen
    pub storm: f32,        // storm strength, 0.0 calm to 1.0 at full, follows the level
    pub sea_storm: f32,    // sea state, like storm but steps to the new level during a level up
    pub wind: f32,         // storm clocks, sped up by the storm so changes stay smooth
    pub rain: f32,
    pub sea: f32,
    pub bolt: f32,     // lightning clock, strikes come faster as the storm builds
    pub whiteout: f32, // 0.0 to 1.0 past a full storm, rain and spray blot out the view
    pub title: f32,    // title backdrop clock
}

impl Background {
    pub fn new() -> Self {
        Self {
            style: BackgroundStyle::Stars,
            time: 0.0,
            energy: 0.0,
            level_up: None,
            stars: 0,
            freeze: None,
            freeze_rate: 0.0,
            frozen_at: 0.0,
            thaw: None,
            twinkle: 0.0,
            storm: 0.0,
            sea_storm: 0.0,
            wind: 0.0,
            rain: 0.0,
            sea: 0.0,
            bolt: 0.0,
            whiteout: 0.0,
            title: 0.0,
        }
    }

    /// step through the styles
    pub fn cycle(&mut self, delta: isize) {
        self.style = crate::style::cycle(&BackgroundStyle::ALL, self.style, delta);
    }

    fn speed(&self, level: usize) -> f32 {
        let level_speed = (1.0 + (level.max(1) - 1) as f32 * LEVEL_SPEEDUP).min(MAX_LEVEL_SPEED);
        level_speed * (1.0 + SURGE_SPEED * self.energy)
    }

    pub fn update(&mut self, delta_time: f64, level: usize) {
        let dt = delta_time as f32;
        self.title += dt;
        let speed = self.speed(level);
        let before = self.time;
        self.time += dt
            * match (self.freeze, self.style) {
                // plasma cells freeze one by one, see frozen_time
                (Some(_), BackgroundStyle::Plasma) => self.freeze_rate,
                (Some(_), _) => self.freeze_rate * (1.0 - self.chill()),
                (None, BackgroundStyle::Plasma) => speed + RUSH_SPEED * self.rush(),
                (None, _) => speed,
            };
        self.twinkle += dt * speed;
        // the storm builds with the level, wind, rain and sea speed up with it
        // front loaded: most of the build comes in the first few levels
        let target = ((level.max(1) - 1) as f32 / STORM_LEVELS)
            .min(1.0)
            .powf(0.7);
        self.storm += (target - self.storm) * (dt * STORM_RAMP).min(1.0);
        // the sea settles into the new level's state as the level up plays out
        let sea_ramp = if self.level_up.is_some() {
            SEA_LEVEL_UP_RAMP
        } else {
            STORM_RAMP
        };
        self.sea_storm += (target - self.sea_storm) * (dt * sea_ramp).min(1.0);
        let level = level.max(1) as f32;
        // creeps in slowly at first, then closes in
        let white = ((level - WHITEOUT_START + 1.0) / WHITEOUT_LEVELS)
            .clamp(0.0, 1.0)
            .powf(1.5);
        self.whiteout += (white - self.whiteout) * (dt * STORM_RAMP).min(1.0);
        let step = self.time - before;
        // a level up heaves the sea
        let rush = self.rush();
        self.wind += step * (1.0 + 3.0 * self.storm);
        self.rain += step * (1.0 + 0.7 * self.storm + 0.5 * self.whiteout);
        self.sea += step * (1.0 + 0.9 * self.storm + STORM_RUSH_SEA * rush);
        self.bolt += step * (1.0 + 1.5 * self.storm);
        self.energy = (self.energy - SURGE_DECAY * dt).max(0.0);
        self.level_up = self.level_up.map(|t| t + dt).filter(|t| *t < LEVEL_UP_TIME);
        match self.thaw {
            Some(t) if t + dt >= THAW_TIME => {
                self.freeze = None;
                self.thaw = None;
            }
            Some(t) => self.thaw = Some(t + dt),
            None => self.freeze = self.freeze.map(|t| t + dt),
        }
    }

    /// update in small steps
    pub fn skip(&mut self, secs: f32, level: usize) {
        for _ in 0..(secs / SKIP_STEP).round() as usize {
            self.update(SKIP_STEP as f64, level);
        }
    }

    /// game over
    pub fn freeze(&mut self, level: usize) {
        self.freeze = Some(0.0);
        self.freeze_rate = self.speed(level);
        self.frozen_at = self.time;
        self.level_up = None;
        self.thaw = None;
    }

    /// start thawing a freeze
    pub fn thaw(&mut self) {
        if self.freeze.is_some() && self.thaw.is_none() {
            self.thaw = Some(0.0);
        }
    }

    /// seconds since game over, held while thawing
    pub fn frozen(&self) -> Option<f32> {
        self.freeze
    }

    /// seconds since the thaw started
    pub fn thawed(&self) -> f32 {
        self.thaw.unwrap_or(0.0)
    }

    /// how stopped, 0.0 moving to 1.0 still
    pub fn chill(&self) -> f32 {
        self.freeze
            .map_or(0.0, |t| smoothstep(t / FREEZE_TIME) * (1.0 - self.melt()))
    }

    /// thaw progress, 0.0 to 1.0
    pub fn melt(&self) -> f32 {
        smoothstep(self.thawed() / THAW_TIME)
    }

    /// clock at a moment after game over
    pub fn frozen_time(&self, after: f32) -> f32 {
        self.frozen_at + after * self.freeze_rate
    }

    pub fn level_up(&mut self) {
        self.level_up = Some(0.0);
        self.stars = (self.stars + 1) % STAR_FIELDS;
    }

    pub fn reset(&mut self) {
        self.energy = 0.0;
        self.level_up = None;
        self.freeze = None;
        self.thaw = None;
    }

    /// level up progress, 0.0 to 1.0
    pub fn wave(&self) -> Option<f32> {
        self.level_up.map(|t| t / LEVEL_UP_TIME)
    }

    /// level up speed burst, 0.0 to 1.0
    pub fn rush(&self) -> f32 {
        self.wave()
            .map_or(0.0, |p| (p * std::f32::consts::PI).sin().powi(2))
    }

    /// line clear surge
    pub fn surge(&mut self, lines: usize) {
        let energy = SURGE[lines.clamp(1, SURGE.len()) - 1];
        self.energy = self.energy.max(energy);
    }
}

fn smoothstep(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
