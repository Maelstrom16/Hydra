pub mod channel;

use std::{cell::RefCell, f32, rc::Rc, sync::{Arc, RwLock}, time::Instant};

use cpal::{OutputCallbackInfo, Sample, Stream};
use ringbuf::{HeapProd, traits::{Observer, Producer}};

use crate::{audio::Audio, common::audio, gameboy::{apu::channel::Pulse, timer::MasterTimer}};

pub struct Apu {       
    div: u8,

    pub(super) pulse1: Rc<RefCell<Pulse>>,
    pub(super) pulse2: Rc<RefCell<Pulse>>,
    // wave: Wave,
    // noise: Noise,

    global_sample_rate: u32,
    local_buffer: Vec<f32>,
    ring_buffer: HeapProd<f32>,
}

impl Apu {
    const SAMPLE_RATE: u32 = MasterTimer::PPU_DOTS_PER_FRAME * 15;

    pub fn new(audio: Arc<RwLock<Audio>>) -> Self {
        let global_sample_rate = audio.read().unwrap().get_sample_rate();
        let ring_buffer = audio.write().unwrap().get_producer();

        Apu { 
            div: 0,
            pulse1: Rc::new(RefCell::new(Pulse::new1())),
            pulse2: Rc::new(RefCell::new(Pulse::new2())),
            // wave: Wave::new(),
            // noise: Noise::new(),

            global_sample_rate,
            local_buffer: Vec::new(),
            ring_buffer,
        }
    }

    /// Tick function to be called on every machine cycle to generate audio samples.
    pub fn system_tick(&mut self) {
        let pulse1_sample = self.pulse1.borrow_mut().tick_and_sample().to_sample::<f32>();
        let pulse2_sample = self.pulse2.borrow_mut().tick_and_sample().to_sample::<f32>();
        let sample = (pulse1_sample + pulse2_sample) / 2.0;
        self.local_buffer.push(sample);
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

    pub fn clone_pointers(&self) -> (Rc<RefCell<Pulse>>, Rc<RefCell<Pulse>>) {
        (self.pulse1.clone(), self.pulse2.clone())
    }
}