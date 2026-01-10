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
    common::bit::MaskedBitSet, gameboy::{
        memory::{Memory, io::{self, GBReg}, vram::Vram},
        ppu::fifo::RenderQueue,
    }, graphics::Graphics, window::UserEvent
};

pub struct PPU {
    fifo: RenderQueue,
    mode: Mode,
    screen_buffer: Box<[u8]>,

    vram: Rc<RefCell<Vram>>,
    lcdc: Rc<GBReg>,
    stat: Rc<GBReg>,
    scy: Rc<GBReg>,
    scx: Rc<GBReg>,
    ly: Rc<GBReg>,
    lyc: Rc<GBReg>,
    wy: Rc<GBReg>,
    wx: Rc<GBReg>,

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
const SCREEN_WIDTH: u8 = 160;
const SCREEN_HEIGHT: u8 = 144;
const BUFFER_SIZE: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4;

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
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
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
                    if ly == SCREEN_HEIGHT {
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

                    // Slight delay in rendering depending on horizontal scroll
                    for _ in 0..(self.scx.get() % 8) {
                        co.yield_(()).await;
                    }

                    // Begin rendering at 
                    for screen_x in 0..SCREEN_WIDTH {
                        let lcdc = self.lcdc.get();
                        // Only render if LCD is enabled (LCDC bit 7)
                        // if lcdc.bit(7) {
                        //     let bg_map_address = if lcdc.bit(3) {0x9C00} else {0x9800}; 
                        //     let win_map_address = if lcdc.bit(6) {0x9C00} else {0x9800};

                        //     let map_x = u8::wrapping_add(screen_x, self.scx.get());
                        //     let tile_x = map_x / 8;
                        //     let map_y = u8::wrapping_add(self.ly.get(), self.scy.get());
                        //     let tile_y = map_y / 8;
                        //     let tile_map_address = bg_map_address + tile_x + (tile_y * 0x10);
                        //     let tile_data_index = self.vram.borrow().unbound_read_u8(tile_map_address, 0);
                        //     let tile_attributes = self.vram.borrow().unbound_read_u8(tile_map_address, 1);
                        //     let tile_data = self.vram.borrow().unbound_read_u8(address, bank);
                            
                        //     starting_bit_mask = starting_bit_mask.rotate_right(1);
                        // }

                        co.yield_(()).await;
                    }

                    // self.vram.borrow().unbound_read_u8(bg_map_addr, 0);
                    // let bg_tile_addr = bg_map_addr;
                    self.screen_buffer[rand::rng().random_range(0..BUFFER_SIZE)] = rand::rng().random_range(0..=255);

                    // Return to HBlank upon completion of the scanline
                    self.mode = Mode::HBlank
                }
            }
            co.yield_(()).await;
        }
    }

    fn push_to_viewport(&self) {
        let graphics = self.graphics.read().unwrap();
        graphics.update_screen_texture(&self.screen_buffer);
        self.proxy.send_event(UserEvent::RedrawRequest).expect("Unable to render Game Boy graphics: Main event loop closed unexpectedly");
    }
}