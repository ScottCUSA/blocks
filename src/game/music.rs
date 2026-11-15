use ggez::{audio::SoundData, Context, GameResult};
use std::io::Cursor;

use super::stretch::{Stretch, Tempo};

/// music source with live tempo, pitch unchanged
pub struct Track {
    data: SoundData,
    player: rodio::Player,
    tempo: Tempo,
}

impl Track {
    pub fn new(ctx: &Context, path: &str) -> GameResult<Self> {
        let data = SoundData::new(ctx, path)?;
        let player = rodio::Player::connect_new(ctx.audio.device().mixer());
        Ok(Track {
            data,
            player,
            tempo: Tempo::new(),
        })
    }

    /// restart from the top
    pub fn play(&self) {
        self.player.clear();
        match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => self
                .player
                .append(Stretch::new(decoder, self.tempo.clone())),
            Err(err) => log::error!("music decode failed: {err}"),
        }
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.clear();
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stopped(&self) -> bool {
        self.player.empty()
    }

    pub fn paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    pub fn set_tempo(&self, tempo: f32) {
        self.tempo.set(tempo);
    }
}
