mod fifo;

use std::{
    cell::Cell,
    ops::Div,
    rc::Rc,
    sync::{Arc, MutexGuard, RwLock},
    thread::{self, JoinHandle},
};

use futures::lock::Mutex;
use rand::Rng;
use winit::window::Window;

use crate::{
    gameboy::{
        memory::{
            Memory,
            io::{self, IO},
        },
        ppu::fifo::RenderQueue,
    },
    graphics::{self, Graphics},
};

pub struct PPU {
    fifo: RenderQueue,
    mode: Mode,
    screen_buffer: Box<[u8]>,

    pub stat: Rc<Cell<u8>>,
    pub scy: Rc<Cell<u8>>,
    pub scx: Rc<Cell<u8>>,
    pub ly: Rc<Cell<u8>>,
    pub lyc: Rc<Cell<u8>>,
    pub wy: Rc<Cell<u8>>,
    pub wx: Rc<Cell<u8>>,

    window: Arc<Window>,
    graphics: Arc<RwLock<Graphics>>,
}

pub enum Mode {
    HBlank,
    VBlank,
    OAMScan,
    Render,
}

const DOTS: u32 = 456;
const SCANLINES: u32 = 154;
const SCREEN_X: u8 = 160;
const SCREEN_Y: u8 = 144;
const BUFFER_SIZE: usize = SCREEN_X as usize * SCREEN_Y as usize * 4;

impl PPU {
    pub fn new(io: &IO, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = PPU {
            fifo: RenderQueue::new(),
            mode: Mode::OAMScan,
            screen_buffer,

            stat: io[io::STAT].clone(),
            scy: io[io::SCY].clone(),
            scx: io[io::SCX].clone(),
            ly: io[io::LY].clone(),
            lyc: io[io::LYC].clone(),
            wy: io[io::WY].clone(),
            wx: io[io::WX].clone(),

            window,
            graphics,
        };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_X as u32, SCREEN_Y as u32);
    }

    #[inline(always)]
    pub fn step(&mut self, clock: &u32) {
        // Update registers
        let ly = (clock / DOTS) as u8;
        self.ly.set(ly);

        // Perform mode-specific behavior
        let lx = (clock % DOTS) as u8;
        match self.mode {
            Mode::HBlank => {
                if ly == SCREEN_Y {
                    self.mode = Mode::VBlank;
                } else if lx == 0 {
                    self.mode = Mode::OAMScan;
                }
            }
            Mode::VBlank => {
                if ly == 0 {
                    self.mode = Mode::OAMScan
                }
            }
            Mode::OAMScan => {
                if lx == 80 {
                    self.mode = Mode::Render;
                } else {
                    // TODO: Whatever OAM Scan is supposed to do
                }
            }
            Mode::Render => {
                // Screen texture generation
                self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);
            }
        }

        // Update and render
        if *clock == 0 {
            let graphics = self.graphics.read().unwrap();
            graphics.update_screen_texture(&self.screen_buffer);
            self.window.request_redraw();
        }
    }
}
