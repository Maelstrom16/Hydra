mod channel;

use std::sync::Arc;

use crate::{audio::Audio, gameboy::apu::channel::Pulse};

pub struct Apu {            
    div: u8,

    pulse1: Pulse,
    pulse2: Pulse,
    // wave: Wave,
    // noise: Noise,
}

impl Apu {
    pub fn new(audio: Arc<Audio>) -> Self {
        Apu { 
            div: 0,
            pulse1: Pulse::new1(&audio),
            pulse2: Pulse::new2(&audio),
            // wave: Wave::new(),
            // noise: Noise::new(),
        }
    }

    pub fn tick(&mut self) {
        self.div = self.div.wrapping_add(1);
    }
}