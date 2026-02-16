pub mod colormap;
mod fifo;
pub mod lcdc;
pub mod state;

use std::{
    cell::{Cell, RefCell}, rc::Rc, sync::{Arc, RwLock}, thread, time::{Duration, Instant}
};

use genawaiter::stack::Co;
use winit::event_loop::EventLoopProxy;

use crate::{
    gameboy::{
        InterruptFlags, memory::vram::Vram, ppu::{colormap::ColorMap, fifo::RenderQueue, lcdc::LcdController, state::PpuState}, timer::MasterTimer
    }, graphics::Graphics, window::UserEvent
};

pub struct Ppu {
    timer: Rc<RefCell<MasterTimer>>,
    fifo: RenderQueue,
    screen_buffer: Box<[u8]>,
    next_frame_instant: Instant,

    vram: Rc<RefCell<Vram>>,
    lcdc: Rc<RefCell<LcdController>>,
    status: Rc<RefCell<PpuState>>,
    scy: Rc<Cell<u8>>,
    scx: Rc<Cell<u8>>,
    wy: Rc<Cell<u8>>,
    wx: Rc<Cell<u8>>,
    color_map: Rc<RefCell<ColorMap>>,

    interrupt_flags: Rc<RefCell<InterruptFlags>>,

    graphics: Arc<RwLock<Graphics>>,
    proxy: EventLoopProxy<UserEvent>
}

#[derive(Copy, Clone, PartialEq)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    Render = 3,
}

impl PpuMode {
    pub const fn as_stat_line_flag(self) -> u8 {
        match self {
            Self::HBlank  => 0b00001000,
            Self::VBlank  => 0b00010000,
            Self::OAMScan => 0b00100000,
            Self::Render  => 0b00000000,
        }
    }
}

const SCANLINES: u32 = 154;
const SCREEN_WIDTH: u8 = 160;
const SCREEN_HEIGHT: u8 = 144;
const MAP_WIDTH: u8 = 32;
const MAP_HEIGHT: u8 = 32;
const BUFFER_SIZE: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4;

impl Ppu {
    pub fn new(vram: Rc<RefCell<Vram>>, lcdc: Rc<RefCell<LcdController>>, status: Rc<RefCell<PpuState>>, timer: Rc<RefCell<MasterTimer>>, interrupt_flags: Rc<RefCell<InterruptFlags>>, scy: Rc<Cell<u8>>, scx: Rc<Cell<u8>>, wy: Rc<Cell<u8>>, wx: Rc<Cell<u8>>, color_map: Rc<RefCell<ColorMap>>, graphics: Arc<RwLock<Graphics>>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = Ppu {
            timer,
            fifo: RenderQueue::new(),
            screen_buffer,
            next_frame_instant: Instant::now(),

            vram,
            lcdc,
            status,
            scy,
            scx,
            wy,
            wx,
            color_map,

            interrupt_flags,

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
    pub async fn coro(&mut self, co: Co<'_, ()>) {
        loop {
            let (lx, ly) = self.status.borrow().get_dot_coords();

            // Perform mode-specific behavior
            match {self.status.borrow().get_mode()} {
                PpuMode::HBlank => {
                    if ly == SCREEN_HEIGHT {
                        self.status.borrow_mut().set_mode(PpuMode::VBlank);
                        self.push_to_viewport();
                    } else if lx == 0 {
                        self.status.borrow_mut().set_mode(PpuMode::OAMScan);
                    }
                }
                PpuMode::VBlank => {
                    if ly == 0 {
                        self.status.borrow_mut().set_mode(PpuMode::OAMScan);
                    }
                }
                PpuMode::OAMScan => {
                    if lx == 80 {
                        self.status.borrow_mut().set_mode(PpuMode::Render);
                    } else {
                        // TODO: Whatever OAM Scan is supposed to do

                    }
                }
                PpuMode::Render => {
                    // Screen texture generation

                    // Slight delay in rendering depending on horizontal scroll
                    for _ in 0..(self.scx.get() % 8) {
                        co.yield_(()).await;
                    }

                    // Begin rendering at 
                    let screen_y = ly;
                    for screen_x in 0..SCREEN_WIDTH {
                        let color_index = if self.lcdc.borrow().lcd_enabled && self.lcdc.borrow().tilemaps_enabled { // TODO: use tilemaps_enabled for priority in CGB mode
                            let is_window = self.lcdc.borrow().window_enabled && screen_x >= self.wx.get() - 7 && screen_y >= self.wy.get();

                            let data_low_address = self.lcdc.borrow().tilemaps_data_area as u16;
                            let (map_address, map_x, map_y) = match is_window {
                                false => (
                                    self.lcdc.borrow().bg_map_area as u16,
                                    screen_x.wrapping_add(self.scx.get()),
                                    screen_y.wrapping_add(self.scy.get()),
                                ),
                                true => (
                                    self.lcdc.borrow().win_map_area as u16,
                                    screen_x - self.wx.get() + 7,
                                    screen_y - self.wy.get(),
                                ),
                            };

                            let map_index_x = (map_x / 8) as u16;
                            let map_index_y = (map_y / 8) as u16;
                            let data_index_address = map_address + map_index_x + (map_index_y * MAP_WIDTH as u16);

                            let data_index = self.vram.borrow().unbound_read_u8(data_index_address, 0);
                            // let tile_attributes = self.vram.borrow().unbound_read_u8(data_index_address, 1); //TODO: Enable on CGB
                            let data_address = if data_index < 0x80 {
                                data_low_address + (data_index as u16 * 16)
                            } else {
                                0x8800 + ((data_index - 0x80) as u16 * 16)
                            };

                            let tile_y = map_y % 8;
                            let byte_address = data_address + (tile_y as u16 * 2);
                            let data = [self.vram.borrow().unbound_read_u8(byte_address, 0),
                                self.vram.borrow().unbound_read_u8(byte_address + 1, 0)]; //TODO: Switch bank based on attributes

                            let tile_x = map_x % 8;
                            let color_index_bits = data.map(|byte| (byte >> (7 - tile_x)) & 1);

                            color_index_bits[1] << 1 | color_index_bits[0]
                        } else {
                            0
                        };

                        // TODO: allow colors to be configured by user
                        let color = self.color_map.borrow().get_tile_color(color_index);
                        let buffer_address = (screen_x as usize + (screen_y as usize * SCREEN_WIDTH as usize)) * 4;
                        self.screen_buffer[buffer_address..buffer_address + 4].copy_from_slice(color);

                        co.yield_(()).await;
                    }

                    // Return to HBlank upon completion of the scanline
                    self.status.borrow_mut().set_mode(PpuMode::HBlank);
                }
                _ => panic!("Invalid PPU mode")
            }
            co.yield_(()).await;
        }
    }

    fn push_to_viewport(&mut self) {
        // Delay thread
        const SECS_PER_FRAME: f64 = 1f64 / 60f64;
        let duration_until_next = self.next_frame_instant.saturating_duration_since(Instant::now());
        thread::sleep(duration_until_next);
        self.next_frame_instant += Duration::from_secs_f64(SECS_PER_FRAME);

        // Send redraw request through event loop proxy
        let graphics = self.graphics.read().unwrap();
        graphics.update_screen_texture(&self.screen_buffer);
        self.proxy.send_event(UserEvent::RedrawRequest).expect("Unable to render Game Boy graphics: Main event loop closed unexpectedly");
    }
}