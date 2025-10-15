use std::{sync::{Arc, RwLock}, thread::{self, JoinHandle}};

use futures::lock::Mutex;
use rand::Rng;
use winit::window::Window;

use crate::{gameboy::memory::Memory, graphics::{self, Graphics}};

pub struct PPU {
    screen_buffer: Box<[u8]>,

    window: Arc<Window>,
    graphics: Arc<RwLock<Graphics>>,
    memory: Arc<RwLock<Memory>>,
}

const DOTS: usize = 456;
const SCANLINES: usize = 154;
const SCREEN_X: usize = 160;
const SCREEN_Y: usize = 144;
const BUFFER_SIZE: usize = SCREEN_X * SCREEN_Y * 4;

impl PPU {
    pub fn new(window: Arc<Window>, graphics: Arc<RwLock<Graphics>>, memory: Arc<RwLock<Memory>>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = PPU { screen_buffer, window, graphics, memory };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_X as u32, SCREEN_Y as u32);
    }

    #[inline(always)]
    pub fn step(&mut self) {
        // // Test texture generation TODO: Remove when finished testing
        for i in 0..BUFFER_SIZE*4 {
            self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);
        }

        // Update and render
            let graphics = self.graphics.read().unwrap();
            graphics.update_screen_texture(&self.screen_buffer);
            self.window.request_redraw();
    }
}