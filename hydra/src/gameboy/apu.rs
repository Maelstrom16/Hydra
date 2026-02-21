pub mod channel;

use std::{cell::RefCell, f32, rc::Rc, sync::{Arc, RwLock}, time::Instant};

use cpal::{OutputCallbackInfo, Sample, Stream};
use ringbuf::{HeapProd, traits::{Observer, Producer}};

use crate::{audio::Audio, common::audio, gameboy::{apu::channel::{Noise, Pulse, PulseType, Wave}, timer::MasterTimer}};

pub struct Apu {       
    div: u8,

    pub(super) pulse1: Rc<RefCell<Pulse>>,
    pub(super) pulse2: Rc<RefCell<Pulse>>,
    pub(super) wave: Rc<RefCell<Wave>>,
    pub(super) noise: Rc<RefCell<Noise>>,

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
            pulse1: Rc::new(RefCell::new(Pulse::new(PulseType::Pulse1))),
            pulse2: Rc::new(RefCell::new(Pulse::new(PulseType::Pulse2))),
            wave: Rc::new(RefCell::new(Wave::new())),
            noise: Rc::new(RefCell::new(Noise::new())),

            global_sample_rate,
            local_buffer: Vec::new(),
            ring_buffer,
        }
    }

    /// Tick function to be called on every machine cycle to generate audio samples.
    pub fn system_tick(&mut self) {
        let pulse1_sample = self.pulse1.borrow_mut().tick_and_sample();
        let pulse2_sample = self.pulse2.borrow_mut().tick_and_sample();
        let _ = self.wave.borrow_mut().tick_and_sample(); // Hacky solution to tick at twice the rate. TODO: Potentially make cleaner?
        let wave_sample = self.wave.borrow_mut().tick_and_sample();
        let noise_sample = self.noise.borrow_mut().tick_and_sample();
        let sample = (pulse1_sample + pulse2_sample + wave_sample + noise_sample) / 4.0;
        self.local_buffer.push(sample);
    }

    /// Tick function to be called on every DIV-APU tick to update audio channel fields.
    pub fn apu_tick(&mut self) {
        self.div = self.div.wrapping_add(1);

        if self.div % 8 == 0 {
            self.pulse1.borrow_mut().envelope_sweep();
            self.pulse2.borrow_mut().envelope_sweep();
        }

        if self.div % 4 == 0 {
            self.pulse1.borrow_mut().period_sweep();
        }

        if self.div % 2 == 0 {
            self.pulse1.borrow_mut().tick_length();
            self.pulse2.borrow_mut().tick_length();
            self.wave.borrow_mut().tick_length();
            self.noise.borrow_mut().tick_length();
        }
    }

    /// Tick function to be called every frame to push to the global ringbuf.
    pub fn frame(&mut self) {
        let samples = self.global_sample_rate as usize * self.local_buffer.len() / Self::SAMPLE_RATE as usize;
        let new_buffer = (0..samples).into_iter().map(|index| self.local_buffer[index * self.local_buffer.len() / samples]).flat_map(|n| [n, n]).collect::<Vec<_>>();
        self.ring_buffer.push_slice(new_buffer.as_slice());
        self.local_buffer.clear();
    }

    pub fn clone_pointers(&self) -> (Rc<RefCell<Pulse>>, Rc<RefCell<Pulse>>, Rc<RefCell<Wave>>, Rc<RefCell<Noise>>) {
        (self.pulse1.clone(), self.pulse2.clone(), self.wave.clone(), self.noise.clone())
    }
}