use std::{sync::{Arc, MutexGuard, RwLock}, thread::{self, JoinHandle}};

use futures::lock::Mutex;
use rand::Rng;
use winit::window::Window;

use crate::{gameboy::memory::Memory, graphics::{self, Graphics}};

pub struct PPU {
    screen_buffer: Box<[u8]>,

    window: Arc<Window>,
    graphics: Arc<RwLock<Graphics>>,
}

const DOTS: usize = 456;
const SCANLINES: usize = 154;
const SCREEN_X: usize = 160;
const SCREEN_Y: usize = 144;
const BUFFER_SIZE: usize = SCREEN_X * SCREEN_Y * 4;

impl PPU {
    pub fn new(window: Arc<Window>, graphics: Arc<RwLock<Graphics>>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = PPU { screen_buffer, window, graphics };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_X as u32, SCREEN_Y as u32);
    }

    #[inline(always)]
    pub fn step(&mut self, clock: &u32) {
        // // Test texture generation TODO: Remove when finished testing
        self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);

        // Update and render
        // if *clock == 0 {
            let graphics = self.graphics.read().unwrap();
            graphics.update_screen_texture(&self.screen_buffer);
            self.window.request_redraw();
        // }
    }
}