use std::{sync::{Arc, RwLock}, thread::{self, JoinHandle}};

use futures::lock::Mutex;
use rand::Rng;

use crate::{common::clockbarrier::ClockBarrier, gameboy::memory::Memory, graphics::{self, Graphics}};

pub struct PPU {
    screen_buffer: Box<[u8]>,

    graphics: Arc<RwLock<Graphics>>,
    memory: Arc<RwLock<Memory>>,
    clock: Arc<ClockBarrier>,
}

const DOTS: usize = 456;
const SCANLINES: usize = 154;
const SCREEN_X: usize = 160;
const SCREEN_Y: usize = 144;
const BUFFER_SIZE: usize = SCREEN_X * SCREEN_Y * 4;

impl PPU {
    pub fn new(graphics: Arc<RwLock<Graphics>>, memory: Arc<RwLock<Memory>>, clock: Arc<ClockBarrier>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = PPU { screen_buffer, graphics, memory, clock };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_X as u32, SCREEN_Y as u32);
    }

    pub fn run(mut self) -> JoinHandle<()>{
        thread::spawn(move || {
            loop {
                // Test texture generation TODO: Remove when finished testing
                self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);

                // Update and render
                if self.clock.new_frame() {
                    let graphics = self.graphics.read().unwrap();
                    graphics.update_screen_texture(&self.screen_buffer);
                    graphics.render();
                }

                self.clock.wait();
            }
        })
    }
}