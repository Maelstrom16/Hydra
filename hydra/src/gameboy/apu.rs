pub mod channel;
pub mod state;

use std::{cell::RefCell, f32, rc::Rc, sync::{Arc, RwLock}, time::Instant};

use cpal::{OutputCallbackInfo, Sample, Stream};
use ringbuf::{HeapProd, traits::{Observer, Producer}};

use crate::{audio::Audio, common::audio, gameboy::{apu::{channel::{Noise, Pulse, PulseType, Wave}, state::ApuState}, memory::MemoryMap, timer::MasterTimer}};

pub struct Apu {
    dot_counter: u8,

    memory: Rc<RefCell<MemoryMap>>,

    pulse1_sample: f32,
    pulse2_sample: f32,
    wave_sample: f32,
    noise_sample: f32,

    global_sample_rate: u32,
    local_buffer: Vec<f32>,
    ring_buffer: HeapProd<f32>,
}

impl Apu {
    const SAMPLE_RATE: u32 = MasterTimer::PPU_DOTS_PER_FRAME * 30;

    pub fn new(memory: Rc<RefCell<MemoryMap>>, audio: Arc<RwLock<Audio>>) -> Self {
        let global_sample_rate = audio.read().unwrap().get_sample_rate();
        let ring_buffer = audio.write().unwrap().get_producer();

        Apu { 
            dot_counter: 0,

            memory,

            pulse1_sample: 0.0,
            pulse2_sample: 0.0,
            wave_sample: 0.0,
            noise_sample: 0.0,

            global_sample_rate,
            local_buffer: Vec::new(),
            ring_buffer,
        }
    }

    /// Tick function to be called on every master cycle to generate audio samples.
    pub fn dot_tick(&mut self, apu_state: &mut ApuState) {
        self.dot_counter = self.dot_counter.wrapping_add(1);
        if self.dot_counter % 2 == 0 {
            self.wave_sample = apu_state.wave.tick_and_sample();
            if self.dot_counter % 4 == 0 {
                self.pulse1_sample = apu_state.pulse1.tick_and_sample();
                self.pulse2_sample = apu_state.pulse2.tick_and_sample();
                if self.dot_counter % 8 == 0 {
                    self.noise_sample = apu_state.noise.tick_and_sample();
                }
            }
            let sample = (self.pulse1_sample + self.pulse2_sample + self.wave_sample + self.noise_sample) / 4.0;

            self.local_buffer.push(sample);
        }
    }

    /// Tick function to be called every frame to push to the global ringbuf.
    pub fn frame(&mut self) {
        let samples = self.global_sample_rate as usize * self.local_buffer.len() / Self::SAMPLE_RATE as usize;
        let new_buffer = (0..samples).into_iter().map(|index| self.local_buffer[index * self.local_buffer.len() / samples]).flat_map(|n| [n, n]).collect::<Vec<_>>();
        self.ring_buffer.push_slice(new_buffer.as_slice());
        self.local_buffer.clear();
    }
}