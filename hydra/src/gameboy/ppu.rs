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
        memory::{Memory, io::{self, deserialized::{RegBgp, RegLcdc, RegLy, RegLyc, RegScx, RegScy, RegStat, RegWx, RegWy}}, vram::{self, Vram}},
        ppu::fifo::RenderQueue,
    }, graphics::Graphics, window::UserEvent
};

pub struct PPU {
    fifo: RenderQueue,
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
    bgp: RegBgp,

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
const MAP_WIDTH: u8 = 32;
const MAP_HEIGHT: u8 = 32;
const BUFFER_SIZE: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4;

impl PPU {
    pub fn new(memory: Rc<RefCell<Memory>>, graphics: Arc<RwLock<Graphics>>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let memguard = memory.borrow();
        let io = memguard.get_io();
        let mut result = PPU {
            fifo: RenderQueue::new(),
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
            bgp: RegBgp::wrap(io.clone_pointer(io::MMIO::BGP)),

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
            match self.stat.get_ppu_mode() {
                // HBlank
                0 => {
                    if ly == SCREEN_HEIGHT {
                        self.stat.set_ppu_mode(Mode::VBlank as u8);
                        self.push_to_viewport();
                    } else if lx == 0 {
                        self.stat.set_ppu_mode(Mode::OAMScan as u8);
                    }
                }
                // VBlank
                1 => {
                    if ly == 0 {
                        self.stat.set_ppu_mode(Mode::OAMScan as u8);
                    }
                }
                // OAM scan
                2 => {
                    if lx == 80 {
                        self.stat.set_ppu_mode(Mode::Render as u8);
                    } else {
                        // TODO: Whatever OAM Scan is supposed to do
                    }
                }
                // Render
                3 => {
                    // Screen texture generation

                    // Slight delay in rendering depending on horizontal scroll
                    for _ in 0..(self.scx.get() % 8) {
                        co.yield_(()).await;
                    }

                    // Begin rendering at 
                    let screen_y = self.ly.get();
                    for screen_x in 0..SCREEN_WIDTH {
                        // Only render if LCD is enabled (LCDC bit 7)
                        if self.lcdc.get_ppu_enabled() {
                            let data_low_address = if self.lcdc.get_tile_data_index() == 1 {0x8000} else {0x9000};
                            let bg_map_address = if self.lcdc.get_bg_map_index() == 0 {0x9800} else {0x9C00};
                            let win_map_address = if self.lcdc.get_win_map_index() == 0 {0x9800} else {0x9C00};

                            let map_x = u8::wrapping_add(screen_x, self.scx.get());
                            let map_y = u8::wrapping_add(screen_y, self.scy.get());
                            let map_index_x = (map_x / 8) as u16;
                            let map_index_y = (map_y / 8) as u16;
                            let data_index_address = bg_map_address + map_index_x + (map_index_y * MAP_WIDTH as u16);

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
                            let color_bits = data.map(|byte| (byte >> (7 - tile_x)) & 1);
                            let color = color_bits[1] << 1 
                                          | color_bits[0];

                            let color = match color {
                                0b00 => self.bgp.get_color0(),
                                0b01 => self.bgp.get_color1(),
                                0b10 => self.bgp.get_color2(),
                                0b11 => self.bgp.get_color3(),
                                _ => panic!("Invalid color")
                            };

                            let color = match color {
                                0b00 => [255, 255, 255, 255],
                                0b01 => [170, 170, 170, 255],
                                0b10 => [85, 85, 85, 255],
                                0b11 => [0, 0, 0, 255],
                                _ => panic!("Invalid color")
                            };

                            let buffer_address = (screen_x as usize + (screen_y as usize * SCREEN_WIDTH as usize)) * 4;
                            self.screen_buffer[buffer_address..buffer_address + 4].copy_from_slice(color.as_slice());
                        }

                        co.yield_(()).await;
                    }

                    // Return to HBlank upon completion of the scanline
                    self.stat.set_ppu_mode(Mode::HBlank as u8);
                }
                _ => panic!("Invalid PPU mode")
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