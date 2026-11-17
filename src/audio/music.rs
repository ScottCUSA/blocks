use std::{io::Cursor, sync::Arc};

use super::stretch::{Stretch, Tempo};

/// a player that is swapped out instead of cleared
/// clear waits on the audio thread, dropping only flags the old one to stop
struct Voice {
    mixer: rodio::mixer::Mixer,
    player: rodio::Player,
    volume: f32,
}

impl Voice {
    fn new(mixer: &rodio::mixer::Mixer) -> Self {
        Voice {
            mixer: mixer.clone(),
            player: rodio::Player::connect_new(mixer),
            volume: 1.0,
        }
    }

    /// an empty, paused player in place of the last
    fn clear(&mut self) {
        let player = rodio::Player::connect_new(&self.mixer);
        player.set_volume(self.volume);
        player.pause();
        self.player = player;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.player.set_volume(volume);
    }
}

/// music source with live tempo, pitch unchanged
pub struct Track {
    data: Arc<[u8]>,
    voice: Voice,
    tempo: Tempo,
}

impl Track {
    pub fn new(mixer: &rodio::mixer::Mixer, data: Arc<[u8]>) -> Self {
        Track {
            data,
            voice: Voice::new(mixer),
            tempo: Tempo::new(),
        }
    }

    /// restart from the top
    pub fn play(&mut self) {
        self.voice.clear();
        match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => self
                .voice
                .player
                .append(Stretch::new(decoder, self.tempo.clone())),
            Err(err) => log::error!("music decode failed: {err}"),
        }
        self.voice.player.play();
    }

    pub fn stop(&mut self) {
        self.voice.clear();
    }

    pub fn pause(&self) {
        self.voice.player.pause();
    }

    pub fn resume(&self) {
        self.voice.player.play();
    }

    pub fn stopped(&self) -> bool {
        self.voice.player.empty()
    }

    pub fn paused(&self) -> bool {
        self.voice.player.is_paused()
    }

    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.voice.set_volume(volume);
    }

    pub fn set_tempo(&self, tempo: f32) {
        self.tempo.set(tempo);
    }
}

/// one shot sound that can be paused, resumed and stopped
pub struct Jingle {
    data: Arc<[u8]>,
    voice: Voice,
}

impl Jingle {
    pub fn new(mixer: &rodio::mixer::Mixer, data: Arc<[u8]>) -> Self {
        Jingle {
            data,
            voice: Voice::new(mixer),
        }
    }

    /// restart from the top
    pub fn play(&mut self) {
        self.voice.clear();
        match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => self.voice.player.append(decoder),
            Err(err) => log::error!("jingle decode failed: {err}"),
        }
        self.voice.player.play();
    }

    pub fn stop(&mut self) {
        self.voice.clear();
    }

    pub fn pause(&self) {
        self.voice.player.pause();
    }

    pub fn resume(&self) {
        self.voice.player.play();
    }

    pub fn playing(&self) -> bool {
        !self.voice.player.is_paused() && !self.voice.player.empty()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.voice.set_volume(volume);
    }
}
