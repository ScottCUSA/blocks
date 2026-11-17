use std::{io::Cursor, sync::Arc};

/// sound effect, a replay cuts off the last one
/// each play gets its own player and the last is dropped, which only flags it to stop
pub struct Effect {
    data: Arc<[u8]>,
    mixer: rodio::mixer::Mixer,
    volume: f32,
    voice: Option<rodio::Player>,
}

impl Effect {
    pub fn new(mixer: &rodio::mixer::Mixer, data: Arc<[u8]>) -> Self {
        Effect {
            data,
            mixer: mixer.clone(),
            volume: 1.0,
            voice: None,
        }
    }

    pub fn play(&mut self) {
        match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => {
                let player = rodio::Player::connect_new(&self.mixer);
                player.set_volume(self.volume);
                player.append(decoder);
                self.voice = Some(player);
            }
            Err(err) => log::error!("sfx decode failed: {err}"),
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(player) = &self.voice {
            player.set_volume(volume);
        }
    }
}
