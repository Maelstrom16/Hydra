pub mod channel;
pub mod state;

use std::{cell::RefCell, f32, rc::Rc, sync::{Arc, RwLock}, time::Instant};

use cpal::{OutputCallbackInfo, Sample, Stream};
use ringbuf::{HeapProd, traits::{Observer, Producer}};

use crate::{audio::Audio, common::audio, gameboy::{apu::{channel::{Noise, Pulse, PulseType, Wave}, state::ApuState}, memory::MemoryMap, timer::MasterTimer}};

pub struct Apu {
    dot_counter: u8,

    global_sample_rate: u32,
    local_buffer_l: Vec<f32>,
    local_buffer_r: Vec<f32>,
    ring_buffer: HeapProd<f32>,
}

impl Apu {
    const SAMPLE_RATE: u32 = MasterTimer::PPU_DOTS_PER_FRAME * 30;

    pub fn new(audio: Arc<RwLock<Audio>>) -> Self {
        let global_sample_rate = audio.read().unwrap().get_sample_rate();
        let ring_buffer = audio.write().unwrap().get_producer();

        Apu { 
            dot_counter: 0,

            global_sample_rate,
            local_buffer_l: Vec::new(),
            local_buffer_r: Vec::new(),
            ring_buffer,
        }
    }

    /// Tick function to be called on every master cycle to generate audio samples.
    pub fn dot_tick(&mut self, apu_state: &mut ApuState) {
        self.dot_counter = self.dot_counter.wrapping_add(1);
        if self.dot_counter % 2 == 0 {
            let [sample_l, sample_r] = apu_state.dot_tick(self.dot_counter);
            self.local_buffer_l.push(sample_l);
            self.local_buffer_r.push(sample_r);
        }
    }

    /// Tick function to be called every frame to push to the global ringbuf.
    pub fn frame(&mut self) {
        let old_sample_count = self.local_buffer_l.len();
        let new_sample_count = self.global_sample_rate as usize * old_sample_count / Self::SAMPLE_RATE as usize;
        
        let new_buffer = (0..new_sample_count).into_iter().map(|index| index * old_sample_count / new_sample_count).flat_map(|new_index| [self.local_buffer_l[new_index], self.local_buffer_r[new_index]]).collect::<Vec<_>>();
        self.ring_buffer.push_slice(new_buffer.as_slice());

        self.local_buffer_l.clear();
        self.local_buffer_r.clear();
    }
}