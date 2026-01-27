mod fifo;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, RwLock},
};

use genawaiter::stack::Co;
use rand::Rng;
use winit::event_loop::EventLoopProxy;

use crate::{
    gameboy::{
        memory::{Memory, io::{self, deserialized::{RegLcdc, RegLy, RegLyc, RegScx, RegScy, RegStat, RegWx, RegWy}}, vram::Vram},
        ppu::fifo::RenderQueue,
    }, graphics::Graphics, window::UserEvent
};

pub struct PPU {
    fifo: RenderQueue,
    mode: Mode,
    screen_buffer: Box<[u8]>,

    vram: Rc<RefCell<Vram>>,
    lcdc: RegLcdc,
    stat: RegStat,
    scy: RegScy,
    scx: RegScx,
    ly: RegLy,
    lyc: RegLyc,
    wy: RegWy,
    wx: RegWx,

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
            lcdc: RegLcdc::wrap(io.clone_pointer(io::MMIO::LCDC)),
            stat: RegStat::wrap(io.clone_pointer(io::MMIO::STAT)),
            scy: RegScy::wrap(io.clone_pointer(io::MMIO::SCY)),
            scx: RegScx::wrap(io.clone_pointer(io::MMIO::SCX)),
            ly: RegLy::wrap(io.clone_pointer(io::MMIO::LY)),
            lyc: RegLyc::wrap(io.clone_pointer(io::MMIO::LYC)),
            wy: RegWy::wrap(io.clone_pointer(io::MMIO::WY)),
            wx: RegWx::wrap(io.clone_pointer(io::MMIO::WX)),

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
                        // Only render if LCD is enabled (LCDC bit 7)
                        if self.lcdc.get_ppu_enabled() {
                            let bg_map_address = if self.lcdc.get_bg_map_index() == 0 {0x9800} else {0x9C00};
                            let win_map_address = if self.lcdc.get_win_map_index() == 0 {0x9800} else {0x9C00};

                            let map_x = u8::wrapping_add(screen_x, self.scx.get());
                            let map_y = u8::wrapping_add(self.ly.get(), self.scy.get());
                            let map_index_x = (map_x / 8) as u16;
                            let map_index_y = (map_y / 8) as u16;
                            let data_pointer_address = bg_map_address + map_index_x + (map_index_y * 0x10);

                            let data_index = self.vram.borrow().unbound_read_u8(data_pointer_address, 0);
                            let tile_attributes = self.vram.borrow().unbound_read_u8(data_pointer_address, 1); //TODO: Disable on DMG
                        }

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