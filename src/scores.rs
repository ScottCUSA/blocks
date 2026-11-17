use serde::{Deserialize, Serialize};

use crate::settings::{data_path, load_toml, save_toml};

pub const MAX_SCORES: usize = 10;
const SCORES_FILE: &str = "scores.toml";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighScore {
    pub initials: String,
    pub score: usize,
    pub level: usize,
    pub lines: usize,
}

/// best scores, highest first
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bevy::prelude::Resource)]
#[serde(default)]
pub struct ScoreBoard {
    pub scores: Vec<HighScore>,
}

impl ScoreBoard {
    pub fn load() -> Self {
        let mut board: ScoreBoard = data_path(SCORES_FILE)
            .and_then(|path| load_toml(&path))
            .unwrap_or_default();
        board.scores.sort_by_key(|s| std::cmp::Reverse(s.score));
        board.scores.truncate(MAX_SCORES);
        board
    }

    pub fn save(&self) {
        if let Some(path) = data_path(SCORES_FILE) {
            save_toml(&path, self);
        }
    }

    pub fn qualifies(&self, score: usize) -> bool {
        score > 0
            && (self.scores.len() < MAX_SCORES
                || self.scores.last().is_some_and(|last| score > last.score))
    }

    /// rank of the new entry, after equal scores
    pub fn insert(&mut self, entry: HighScore) -> Option<usize> {
        if !self.qualifies(entry.score) {
            return None;
        }
        let rank = self
            .scores
            .iter()
            .position(|s| s.score < entry.score)
            .unwrap_or(self.scores.len());
        self.scores.insert(rank, entry);
        self.scores.truncate(MAX_SCORES);
        Some(rank)
    }

    pub fn best(&self) -> usize {
        self.scores.first().map_or(0, |s| s.score)
    }
}

/// arcade style initials being entered, up to MAX_INITIALS letters
#[derive(Debug, Clone, PartialEq, bevy::prelude::Resource)]
pub struct Initials {
    pub letters: Vec<u8>,
    pub cursor: usize,
}

pub const MAX_INITIALS: usize = 3;

impl Initials {
    /// from saved initials, empty if invalid
    pub fn new(saved: &str) -> Self {
        let valid = saved.len() <= MAX_INITIALS && saved.bytes().all(|b| b.is_ascii_uppercase());
        let letters = if valid {
            saved.bytes().collect()
        } else {
            Vec::new()
        };
        Self { letters, cursor: 0 }
    }

    /// step the letter under the cursor, wrapping, an empty cell starts at A or Z
    pub fn step(&mut self, delta: i8) {
        if self.cursor == self.letters.len() {
            self.letters.push(if delta > 0 { b'A' } else { b'Z' });
            return;
        }
        let letter = &mut self.letters[self.cursor];
        *letter = b'A' + ((*letter - b'A') as i8 + delta).rem_euclid(26) as u8;
    }

    /// within the letters and the first empty cell
    pub fn move_cursor(&mut self, delta: isize) {
        let last = self.letters.len().min(MAX_INITIALS - 1);
        self.cursor = (self.cursor as isize + delta).clamp(0, last as isize) as usize;
    }

    /// set the letter under the cursor and advance
    pub fn type_letter(&mut self, letter: char) {
        if letter.is_ascii_alphabetic() {
            let letter = letter.to_ascii_uppercase() as u8;
            match self.letters.get_mut(self.cursor) {
                Some(slot) => *slot = letter,
                None => self.letters.push(letter),
            }
            self.move_cursor(1);
        }
    }

    /// delete the letter under the cursor, or before it on an empty cell
    pub fn backspace(&mut self) {
        if self.cursor == self.letters.len() {
            if self.cursor == 0 {
                return;
            }
            self.cursor -= 1;
        }
        self.letters.remove(self.cursor);
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.letters).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(score: usize) -> HighScore {
        HighScore {
            initials: "AAA".into(),
            score,
            level: 1,
            lines: 0,
        }
    }

    #[test]
    fn insert_orders_and_truncates() {
        let mut board = ScoreBoard::default();
        assert!(!board.qualifies(0));
        for score in (1..=MAX_SCORES).map(|i| i * 100) {
            assert!(board.insert(entry(score)).is_some());
        }
        assert_eq!(board.best(), 1000);
        assert!(!board.qualifies(100));
        // ties go after the existing score
        assert_eq!(board.insert(entry(500)), Some(6));
        assert_eq!(board.scores.len(), MAX_SCORES);
        assert_eq!(board.scores.last().unwrap().score, 200);
        assert_eq!(board.insert(entry(5000)), Some(0));
        assert_eq!(board.insert(entry(50)), None);
    }

    #[test]
    fn initials() {
        assert_eq!(Initials::new("abc").text(), "");
        assert_eq!(Initials::new("AB").text(), "AB");
        let mut initials = Initials::new("XYZ");
        initials.step(-1);
        assert_eq!(initials.text(), "WYZ");
        initials.move_cursor(2);
        initials.step(1);
        assert_eq!(initials.text(), "WYA");
        initials.move_cursor(-5);
        initials.type_letter('q');
        assert_eq!((initials.text().as_str(), initials.cursor), ("QYA", 1));
    }

    #[test]
    fn initials_backspace() {
        let mut initials = Initials::new("");
        initials.backspace();
        initials.type_letter('a');
        initials.type_letter('b');
        assert_eq!((initials.text().as_str(), initials.cursor), ("AB", 2));
        // cursor stops at the first empty cell
        initials.move_cursor(5);
        assert_eq!(initials.cursor, 2);
        initials.backspace();
        assert_eq!((initials.text().as_str(), initials.cursor), ("A", 1));
        initials.step(-1);
        assert_eq!(initials.text(), "AZ");
        initials.type_letter('c');
        initials.type_letter('d');
        assert_eq!((initials.text().as_str(), initials.cursor), ("ACD", 2));
        initials.backspace();
        assert_eq!((initials.text().as_str(), initials.cursor), ("AC", 2));
        initials.move_cursor(-2);
        initials.backspace();
        assert_eq!((initials.text().as_str(), initials.cursor), ("C", 0));
    }

    #[test]
    fn round_trip() {
        let mut board = ScoreBoard::default();
        board.insert(entry(300));
        board.insert(entry(900));
        let text = toml::to_string(&board).unwrap();
        assert_eq!(toml::from_str::<ScoreBoard>(&text).unwrap(), board);
    }
}
