mod channel;

use std::{f32, sync::{Arc, RwLock}, time::Instant};

use cpal::{OutputCallbackInfo, Stream};
use ringbuf::{HeapProd, traits::{Observer, Producer}};

use crate::{audio::Audio, common::audio, gameboy::{apu::channel::Pulse, timer::MasterTimer}};

pub struct Apu {       
    div: u8,

    pulse1: Pulse,
    pulse2: Pulse,
    // wave: Wave,
    // noise: Noise,

    global_sample_rate: u32,
    local_buffer: Vec<f32>,
    ring_buffer: HeapProd<f32>,

    phase_accumulator: f32
}

impl Apu {
    const SAMPLE_RATE: u32 = MasterTimer::PPU_DOTS_PER_FRAME * 60;

    pub fn new(audio: Arc<RwLock<Audio>>) -> Self {
        let global_sample_rate = audio.read().unwrap().get_sample_rate();
        let ring_buffer = audio.write().unwrap().get_producer();

        Apu { 
            div: 0,
            pulse1: Pulse::new1(),
            pulse2: Pulse::new2(),
            // wave: Wave::new(),
            // noise: Noise::new(),

            global_sample_rate,
            local_buffer: Vec::new(),
            ring_buffer,

            phase_accumulator: 0.0
        }
    }

    /// Tick function to be called on every machine cycle to generate audio samples.
    pub fn system_tick(&mut self) {
        let inc = (440.0 * f32::consts::TAU) / Self::SAMPLE_RATE as f32;
        self.phase_accumulator = (self.phase_accumulator + inc) % f32::consts::TAU;
        self.local_buffer.push(self.phase_accumulator.sin());
    }

    /// Tick function to be called on every DIV-APU tick to update audio channel fields.
    pub fn apu_tick(&mut self) {
        self.div = self.div.wrapping_add(1);
    }

    /// Tick function to be called every frame to push to the global ringbuf.
    pub fn frame(&mut self) {
        let samples = self.global_sample_rate as usize * self.local_buffer.len() / Self::SAMPLE_RATE as usize;
        let new_buffer = (0..samples).into_iter().map(|index| self.local_buffer[index * self.local_buffer.len() / samples]).flat_map(|n| [n, n]).collect::<Vec<_>>();
        self.ring_buffer.push_slice(new_buffer.as_slice());
        self.local_buffer.clear();
    }
}