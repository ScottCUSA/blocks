use ggez::{audio::SoundData, Context, GameResult};
use std::io::Cursor;

use super::stretch::{Stretch, Tempo};

/// music source with live tempo, pitch unchanged
/// Player::clear blocks until the audio thread drains it (~15-40ms),
/// so each play gets its own player and the last is dropped, which only flags it to stop
pub struct Track {
    data: SoundData,
    mixer: rodio::mixer::Mixer,
    volume: f32,
    player: Option<rodio::Player>,
    tempo: Option<Tempo>, // None plays as is
}

impl Track {
    pub fn new(ctx: &Context, path: &str) -> GameResult<Self> {
        let mut track = Self::plain(ctx, path)?;
        track.tempo = Some(Tempo::new());
        Ok(track)
    }

    /// no tempo control
    pub fn plain(ctx: &Context, path: &str) -> GameResult<Self> {
        Ok(Track {
            data: SoundData::new(ctx, path)?,
            mixer: ctx.audio.device().mixer().clone(),
            volume: 1.0,
            player: None,
            tempo: None,
        })
    }

    /// restart from the top
    pub fn play(&mut self) {
        let decoder = match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => decoder,
            Err(err) => {
                log::error!("music decode failed: {err}");
                return;
            }
        };
        let player = rodio::Player::connect_new(&self.mixer);
        player.set_volume(self.volume);
        match &self.tempo {
            Some(tempo) => player.append(Stretch::new(decoder, tempo.clone())),
            None => player.append(decoder),
        }
        self.player = Some(player);
    }

    pub fn stop(&mut self) {
        self.player = None;
    }

    pub fn pause(&self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }

    pub fn resume(&self) {
        if let Some(player) = &self.player {
            player.play();
        }
    }

    pub fn stopped(&self) -> bool {
        self.player.as_ref().is_none_or(rodio::Player::empty)
    }

    pub fn paused(&self) -> bool {
        self.player.as_ref().is_some_and(rodio::Player::is_paused)
    }

    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(player) = &self.player {
            player.set_volume(volume);
        }
    }

    pub fn set_tempo(&self, tempo: f32) {
        if let Some(t) = &self.tempo {
            t.set(tempo);
        }
    }
}
