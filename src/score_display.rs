const ROLL_TIME: f64 = 0.5; // seconds for the shown score to count up
const POP_TIME: f64 = 0.35; // seconds the score stays enlarged after increasing
const POPUP_TIME: f64 = 1.0; // seconds a "+points" popup floats up

pub struct ScorePopup {
    pub points: usize,
    time: f64,
}

impl ScorePopup {
    /// fraction of the popup's life, 0.0 to 1.0
    pub fn progress(&self) -> f32 {
        (self.time / POPUP_TIME).min(1.0) as f32
    }
}

/// animated view of the score
pub struct ScoreDisplay {
    from: usize,
    target: usize,
    time: f64, // seconds since the last increase
    pub popups: Vec<ScorePopup>,
}

impl ScoreDisplay {
    pub fn new(score: usize) -> Self {
        Self {
            from: score,
            target: score,
            time: ROLL_TIME.max(POP_TIME),
            popups: Vec::new(),
        }
    }

    pub fn add(&mut self, points: usize) {
        self.from = self.shown();
        self.target += points;
        self.time = 0.0;
        self.popups.push(ScorePopup { points, time: 0.0 });
    }

    pub fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
        for popup in &mut self.popups {
            popup.time += delta_time;
        }
        self.popups.retain(|popup| popup.time < POPUP_TIME);
    }

    /// the score counting up towards the real score
    pub fn shown(&self) -> usize {
        let t = (self.time / ROLL_TIME).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.from + ((self.target - self.from) as f64 * eased).round() as usize
    }

    /// 1.0 right after an increase, falling to 0.0
    pub fn pop(&self) -> f32 {
        (1.0 - self.time / POP_TIME).max(0.0) as f32
    }
}
