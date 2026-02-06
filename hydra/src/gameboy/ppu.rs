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
        memory::{MemoryMap, io::{self, IOMap, deserialized::{RegBgp, RegIf, RegLcdc, RegLy, RegLyc, RegScx, RegScy, RegStat, RegWx, RegWy}}, vram::{self, Vram}},
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

    r#if: RegIf,

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
    pub fn new(vram: Rc<RefCell<Vram>>, io: &IOMap, graphics: Arc<RwLock<Graphics>>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let screen_buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let mut result = PPU {
            fifo: RenderQueue::new(),
            screen_buffer,

            vram,
            lcdc: RegLcdc::wrap(io.clone_pointer(io::MMIO::LCDC)),
            stat: RegStat::wrap(io.clone_pointer(io::MMIO::STAT)),
            scy: RegScy::wrap(io.clone_pointer(io::MMIO::SCY)),
            scx: RegScx::wrap(io.clone_pointer(io::MMIO::SCX)),
            ly: RegLy::wrap(io.clone_pointer(io::MMIO::LY)),
            lyc: RegLyc::wrap(io.clone_pointer(io::MMIO::LYC)),
            wy: RegWy::wrap(io.clone_pointer(io::MMIO::WY)),
            wx: RegWx::wrap(io.clone_pointer(io::MMIO::WX)),
            bgp: RegBgp::wrap(io.clone_pointer(io::MMIO::BGP)),

            r#if: RegIf::wrap(io.clone_pointer(io::MMIO::IF)),

            graphics,
            proxy,
        };
        result.init_graphics();
        result
    }

    fn init_graphics(&mut self) {
        self.graphics.write().unwrap().resize_screen_texture(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    }

    fn set_ppu_mode(&mut self, mode: Mode) {
        if let Mode::VBlank = mode {self.r#if.set_vblank(true)};
        self.stat.set_ppu_mode(mode as u8);
    }

    fn update_ly(&mut self, clk: u32) -> u8 {
        let ly = (clk / DOTS) as u8;
        self.ly.set(ly);
        self.stat.set_ly_equals_lyc(ly == self.lyc.get());

        ly
    }

    fn stat_interrupt(&mut self) {
        let ppu_mode = self.stat.get_ppu_mode();
        if self.stat.get_mode_0_int() && ppu_mode == 0
        || self.stat.get_mode_1_int() && ppu_mode == 1
        || self.stat.get_mode_2_int() && ppu_mode == 2
        || self.stat.get_lyc_int() && self.stat.get_ly_equals_lyc() {
            self.r#if.set_stat(true);
        }
    }

    #[inline(always)]
    pub async fn coro(&mut self, clock: Rc<Cell<u32>>, co: Co<'_, ()>) {
        loop {
            // Update screen position
            let clk = clock.get();
            let ly = self.update_ly(clk);
            let lx = (clk % DOTS) as u8;

            self.stat_interrupt(); // TODO: Verify when this needs to be called

            // Perform mode-specific behavior
            match self.stat.get_ppu_mode() {
                // HBlank
                0 => {
                    if ly == SCREEN_HEIGHT {
                        self.set_ppu_mode(Mode::VBlank);
                        self.push_to_viewport();
                    } else if lx == 0 {
                        self.set_ppu_mode(Mode::OAMScan);
                    }
                }
                // VBlank
                1 => {
                    if ly == 0 {
                        self.set_ppu_mode(Mode::OAMScan);
                    }
                }
                // OAM scan
                2 => {
                    if lx == 80 {
                        self.set_ppu_mode(Mode::Render);
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
                            let color_index_bits = data.map(|byte| (byte >> (7 - tile_x)) & 1);
                            let color_index = color_index_bits[1] << 1 
                                          | color_index_bits[0];

                            // TODO: allow colors to be configured by user
                            let color = self.bgp.get_color(color_index);

                            let buffer_address = (screen_x as usize + (screen_y as usize * SCREEN_WIDTH as usize)) * 4;
                            self.screen_buffer[buffer_address..buffer_address + 4].copy_from_slice(color);
                        }

                        co.yield_(()).await;
                    }

                    // Return to HBlank upon completion of the scanline
                    self.set_ppu_mode(Mode::HBlank);
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