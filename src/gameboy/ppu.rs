mod fifo;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, RwLock},
    time::Instant,
};

use genawaiter::stack::Co;
use rand::Rng;
use winit::event_loop::EventLoopProxy;

use crate::{
    common::bit::BitSet, gameboy::{
        memory::{Memory, io::{self, IOReg}, vram::Vram},
        ppu::fifo::RenderQueue,
    }, graphics::Graphics, window::UserEvent
};

pub struct PPU {
    fifo: RenderQueue,
    mode: Mode,
    screen_buffer: Box<[u8]>,

    vram: Rc<RefCell<Vram>>,
    lcdc: Rc<IOReg>,
    stat: Rc<IOReg>,
    scy: Rc<IOReg>,
    scx: Rc<IOReg>,
    ly: Rc<IOReg>,
    lyc: Rc<IOReg>,
    wy: Rc<IOReg>,
    wx: Rc<IOReg>,

    graphics: Arc<RwLock<Graphics>>,
    proxy: EventLoopProxy<UserEvent>
}

pub enum Mode {
    HBlank,
    VBlank,
    OAMScan,
    Render,
}

pub const DOTS: u32 = 456;
const SCANLINES: u32 = 154;
const SCREEN_X: u8 = 160;
const SCREEN_Y: u8 = 144;
const BUFFER_SIZE: usize = SCREEN_X as usize * SCREEN_Y as usize * 4;

impl PPU {
    pub fn new(memory: Rc<RefCell<Memory>>, graphics: Arc<RwLock<Graphics>>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let memguard = memory.borrow();
        let io = memguard.get_io();
        let mut result = PPU {
            fifo: RenderQueue::new(),
            mode: Mode::OAMScan,
            screen_buffer,

            vram: memguard.get_vram(),
            lcdc: io[io::LCDC].clone(),
            stat: io[io::STAT].clone(),
            scy: io[io::SCY].clone(),
            scx: io[io::SCX].clone(),
            ly: io[io::LY].clone(),
            lyc: io[io::LYC].clone(),
            wy: io[io::WY].clone(),
            wx: io[io::WX].clone(),

            graphics,
            proxy,
        };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_X as u32, SCREEN_Y as u32);
    }

    #[inline(always)]
    pub async fn coro(&mut self, clock: Rc<Cell<u32>>, co: Co<'_, ()>) {
        loop {
            // Update screen position
            let clk = clock.get();
            let ly = (clk / DOTS) as u8;
            self.ly.set(ly);
            let lx = (clk % DOTS) as u8;

            // Perform mode-specific behavior
            match self.mode {
                Mode::HBlank => {
                    if ly == SCREEN_Y {
                        self.mode = Mode::VBlank;
                        self.push_to_viewport();
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
                    
                    // Check whether rendering is enabled (LCDC bit 7)
                    let lcdc = self.lcdc.get();
                    let bg_map_addr = if lcdc.bit(3) {0x9C00} else {0x9800}; 
                    let win_map_addr = if lcdc.bit(6) {0x9C00} else {0x9800}; 

                    // Slight delay in rendering depending on horizontal scroll
                    for _ in 0..(self.scx.get() % 8) {
                        co.yield_(()).await;
                    }
                    self.vram.borrow().unbound_read_u8(0x1000, 0);
                    let bg_tile_addr = bg_map_addr;
                    self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);
                }
            }
            co.yield_(()).await;
        }
    }

    fn push_to_viewport(&self) {
        let t1 = Instant::now();
        let graphics = self.graphics.read().unwrap();
        let t2 = Instant::now();
        graphics.update_screen_texture(&self.screen_buffer);
        let t3 = Instant::now();
        self.proxy.send_event(UserEvent::RedrawRequest).expect("Unable to render Game Boy graphics: Main event loop closed unexpectedly");
        let t4 = Instant::now();
    }
}