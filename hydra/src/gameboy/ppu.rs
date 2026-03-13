pub mod attributes;
pub mod colormap;
pub mod fifo;
pub mod state;

use std::{
    cell::{Cell, RefCell}, collections::VecDeque, rc::Rc, sync::{Arc, RwLock}, thread, time::{Duration, Instant}
};

use winit::event_loop::EventLoopProxy;

use crate::{
    gameboy::{
        GbMode, Model, memory::{MemoryMap, oam::{Oam, ObjectOamMetadata}, vram::Vram}, ppu::{attributes::TileAttributes, colormap::{Color, ColorMap}, fifo::FifoFetcher, state::{ObjectHeight, PpuState}}, timer::MasterTimer
    }, graphics::Graphics, window::UserEvent
};

pub struct Ppu {
    model: Rc<Model>,
    fifo: FifoFetcher,
}

#[repr(u8)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan{current_address: u16, obj_meta: Option<ObjectOamMetadata>} = 2,
    Render = 3,
}

impl PpuMode {
    pub const fn default_oam() -> Self {
        PpuMode::OAMScan { current_address: 0xFE00, obj_meta: None }
    }

    pub const fn as_u2(&self) -> u8 {
        match self {
            Self::HBlank => 0b00,
            Self::VBlank => 0b01,
            Self::OAMScan{..} => 0b10,
            Self::Render => 0b11,
        }
    }

    pub const fn as_stat_line_flag(&self) -> u8 {
        match self {
            Self::HBlank => 0b00001000,
            Self::VBlank => 0b00010000,
            Self::OAMScan{..} => 0b00100000,
            Self::Render => 0b00000000,
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
    pub fn new(model: Rc<Model>) -> Self {
        Ppu {
            model,
            fifo: FifoFetcher::new(),
        }
    }

    #[inline(always)]
    pub fn coro(&mut self, memory: &mut MemoryMap) {
        // Do nothing if PPU is disabled
        if !memory.ppu_state.lcd_enabled {return;}

        let (lx, ly) = memory.ppu_state.get_dot_coords();

        // Perform mode-specific behavior
        match memory.ppu_state.ppu_mode {
            PpuMode::HBlank => {
                if ly == SCREEN_HEIGHT {
                    memory.ppu_state.set_mode(PpuMode::VBlank, &mut memory.interrupt_flags);
                    memory.ppu_state.push_to_viewport();
                } else if lx == 0 {
                    memory.ppu_state.set_mode(PpuMode::default_oam(), &mut memory.interrupt_flags);
                    self.fifo.scanline_objects.clear();
                }
            }
            PpuMode::VBlank => {
                if ly == 0 {
                    memory.ppu_state.set_mode(PpuMode::default_oam(), &mut memory.interrupt_flags);
                    self.fifo.scanline_objects.clear();
                }
            }
            PpuMode::OAMScan{ref mut current_address, ref mut obj_meta} => {
                match obj_meta.take() {
                    None if self.fifo.scanline_objects.len() < 10 => {
                        *obj_meta = Some(memory.oam.get_oam_meta(*current_address))
                    }
                    Some(obj) => {
                        if obj.occupies_y(ly, memory.ppu_state.object_size) {
                            self.fifo.scanline_objects.push(obj);
                        }
                    }
                    _ => {/* Object limit reached--just stall */}
                }
                *current_address += 2; // A little hacky, but allows proper spacing

                // Update mode when complete
                if *current_address > 0xFE9F {
                    // No need to sort for CGB, because objects will already be in OAM order
                    if self.model.is_monochrome() {
                        self.fifo.scanline_objects.sort_by(|obj1, obj2| obj1.x.cmp(&obj2.x));
                    }
                    memory.ppu_state.set_mode(PpuMode::Render, &mut memory.interrupt_flags);
                }
            }
            PpuMode::Render => {
                // Screen texture generation

                let (screen_x, screen_y) = (self.fifo.screen_x, ly);
                let color = self.fifo.resolve_color(memory);

                let buffer_address = (screen_x as usize + (screen_y as usize * SCREEN_WIDTH as usize)) * 4;
                memory.ppu_state.screen_buffer[buffer_address..buffer_address + 4].copy_from_slice(&color);

                // Return to HBlank upon completion of the scanline
                if self.fifo.screen_x == 0 {
                    memory.ppu_state.set_mode(PpuMode::HBlank, &mut memory.interrupt_flags);
                }
            }
        }
    }
}