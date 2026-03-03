mod mbc;
pub mod oam;
pub mod rom;
mod sram;
pub mod vram;
pub mod wram;

use crate::{
    common::{errors::HydraIOError, util::Accessors},
    gameboy::{
        GbMode, Model, apu::{Apu, channel::{Noise, Pulse, PulseType, Wave}, state::ApuState}, interrupt::{InterruptEnable, InterruptFlags}, joypad::Joypad, memory::{oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{PpuMode, colormap::{self, ColorMap}, state::PpuState}, serial::SerialConnection, timer::MasterTimer
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc, time::Duration};

pub struct MemoryMap {
    model: Rc<Model>,
    mode: Rc<GbMode>,

    cartridge: Option<Box<dyn mbc::MemoryBankController>>,
    pub(super) vram: Vram,
    wram: Wram,
    pub(super) oam: Oam,

    pub(super) joypad: Joypad,
    pub(super) serial: SerialConnection,
    pub(super) timer: MasterTimer,
    pub(super) interrupt_flags: InterruptFlags,
    pub(super) apu_state: ApuState,
    pub(super) ppu_state: PpuState,
    pub(super) color_map: Box<dyn ColorMap>,

    dma_source: u8,
    dma_cycle: Option<u8>,

    hdma_source: u16,
    hdma_destination: u16,
    hdma_length: u8,

    hram: [u8; 0x7F],
    pub(super) interrupt_enable: InterruptEnable,
}

impl MemoryMap {
    pub fn new(model: Rc<Model>, mode: Rc<GbMode>) -> Result<MemoryMap, HydraIOError> {
        let interrupt_flags = InterruptFlags::new();
        let interrupt_enable = InterruptEnable::new();
        let joypad = Joypad::new();
        let serial = SerialConnection::new(model.clone());
        let vram = Vram::new(model.clone());
        let wram = Wram::new(model.clone());
        let ppu_state = PpuState::new(&model);
        let timer = MasterTimer::new(model.clone(), mode.clone());
        let color_map = colormap::from_mode(&mode);
        let oam = Oam::new(model.clone());
        let apu_state = ApuState::new();
        let dma_source = match model.is_monochrome() {
            true => 0xFF,
            false => 0x00,
        };

        Ok(MemoryMap {
            model,
            mode,

            cartridge: None,
            vram,
            wram,
            oam,

            joypad,
            serial,
            timer,
            interrupt_flags,
            apu_state,
            ppu_state,
            color_map,

            dma_source,
            dma_cycle: None,

            hdma_source: 0xFFF0,
            hdma_destination: 0x1FF0,
            hdma_length: 0xFF,

            hram: [0; 0x7F],
            interrupt_enable,
        })
    }

    pub fn hot_swap_rom(&mut self, rom: Rom,) -> Result<(), HydraIOError> {
        self.cartridge = Some(rom.into_mbc()?);
        Ok(())
    }

    fn is_cart_accessible(&self) -> bool {
        // Only when not performing OAM DMA (unless transferring from WRAM on GBC)
        matches!(self.dma_cycle, None) || (self.model.is_color() && !(0x0000..=0x7FFF).contains(&((self.dma_source as u16) << 8)))
    }

    fn is_vram_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3 when LCD is enabled
        !matches!(self.ppu_state.get_mode(), PpuMode::Render) || !self.ppu_state.is_lcd_enabled()
    }

    fn is_wram_accessible(&self) -> bool {
        // Only when not performing OAM DMA (unless transferring from cartridge on GBC)
        matches!(self.dma_cycle, None) || (self.model.is_color() && !(0xC000..=0xDFFF).contains(&((self.dma_source as u16) << 8)))
    }

    pub fn tick_dma(&mut self) {
        if let Some(cycle) = self.dma_cycle {
            let source_address = (self.dma_source as u16) << 8 | cycle as u16;
            let destination_address = 0xFE00 | cycle as u16;
            self.write_u8(self.read_u8(source_address, true), destination_address, true);

            let next_dma_cycle = cycle + 1;
            self.dma_cycle = if next_dma_cycle < 160 {
                Some(next_dma_cycle)
            } else {
                None
            };
        }
    }

    pub fn read_u8(&self, address: u16, is_dma: bool) -> u8 {
        let read_result = match address {
            0x0000..=0x7FFF if is_dma || self.is_cart_accessible() => self.cartridge.as_ref().map(|this| this.read_rom_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF if self.is_vram_accessible() => self.vram.read_u8(address),
            0xA000..=0xBFFF if is_dma || self.is_cart_accessible() => self.cartridge.as_ref().map(|this| this.read_ram_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF if is_dma || self.is_wram_accessible() => Ok(self.wram.read_u8(address)),
            0xE000..=0xFDFF if is_dma || self.is_wram_accessible() => Ok(self.wram.read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.read(address),
            0xFF00 => Ok(self.joypad.read_joyp()),
            0xFF01..=0xFF02 => self.serial.read(address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.read(address),
            0xFF0F => self.interrupt_flags.read(address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F => self.apu_state.read(address),
            0xFF40..=0xFF45 | 0xFF4A..=0xFF4B => self.ppu_state.read(address),
            0xFF46 => Ok(self.dma_source),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.read(address),
            0xFF4F if matches!(*self.mode, GbMode::CGB) => Ok(self.vram.read_vbk()),
            0xFF51 if matches!(*self.mode, GbMode::CGB) => Ok(0xFF),
            0xFF52 if matches!(*self.mode, GbMode::CGB) => Ok(0xFF),
            0xFF53 if matches!(*self.mode, GbMode::CGB) => Ok(0xFF),
            0xFF54 if matches!(*self.mode, GbMode::CGB) => Ok(0xFF),
            0xFF55 if matches!(*self.mode, GbMode::CGB) => Ok(self.hdma_length),
            0xFF70 if matches!(*self.mode, GbMode::CGB) => Ok(self.wram.read_wbk()),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => self.interrupt_enable.read(address),
            _ => Err(HydraIOError::OpenBusAccess)
        };

        match read_result {
            Ok(value) => value,
            Err(HydraIOError::OpenBusAccess) => {
                println!("Warning: Read from open bus at address {:#06X}", address);
                if let Some(_) = self.dma_cycle {
                    println!("note: DMA active");
                }
                0xFF
            }
            Err(e) => panic!("Error reading from memory.\n{}", e),
        }
    }

    pub fn write_u8(&mut self, val: u8, address: u16, is_dma: bool) -> () {
        let write_result = match address {
            0x0000..=0x7FFF if is_dma || self.is_cart_accessible() => self.cartridge.as_mut().map(|this| this.write_rom_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF if self.is_vram_accessible() => self.vram.write_u8(val, address),
            0xA000..=0xBFFF if is_dma || self.is_cart_accessible() => self.cartridge.as_mut().map(|this| this.write_ram_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF if is_dma || self.is_wram_accessible() => Ok(self.wram.write_u8(val, address)),
            0xE000..=0xFDFF if is_dma || self.is_wram_accessible() => Ok(self.wram.write_u8(val, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.write(address, val),
            0xFF00 => Ok(self.joypad.write_joyp(val, &mut self.interrupt_flags)),
            0xFF01..=0xFF02 => self.serial.write(val, address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.write(val, address, &mut self.apu_state),
            0xFF0F => self.interrupt_flags.write(val, address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F => self.apu_state.write(val, address),
            0xFF40..=0xFF45 | 0xFF4A..=0xFF4B => self.ppu_state.write(val, address, &mut self.interrupt_flags),
            0xFF46 => Ok({self.dma_source = val; self.dma_cycle = Some(0);}),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.write(val, address),
            0xFF4F if matches!(*self.mode, GbMode::CGB) => Ok(self.vram.write_vbk(val)),
            0xFF51 if matches!(*self.mode, GbMode::CGB) => Ok(self.hdma_source = (self.hdma_source & 0xFF) | ((val as u16) << 8)),
            0xFF52 if matches!(*self.mode, GbMode::CGB) => Ok(self.hdma_source = (self.hdma_source & 0xFF00) | (val as u16)),
            0xFF53 if matches!(*self.mode, GbMode::CGB) => Ok(self.hdma_destination = (self.hdma_destination & 0xFF) | ((val as u16 & 0x1F) << 8)),
            0xFF54 if matches!(*self.mode, GbMode::CGB) => Ok(self.hdma_destination = (self.hdma_destination & 0xFF00) | (val as u16)),
            0xFF55 if matches!(*self.mode, GbMode::CGB) => Ok({self.hdma_length = val; self.hdma();}),
            0xFF70 if matches!(*self.mode, GbMode::CGB) => Ok(self.wram.write_wbk(val)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = val),
            0xFFFF => self.interrupt_enable.write(val, address),
            _ => Err(HydraIOError::OpenBusAccess)
        };

        match write_result {
            Ok(_) => {}
            Err(HydraIOError::OpenBusAccess) => {}//println!("Warning: Write to open bus at address {:#06X}", address),
            Err(e) => panic!("Error writing to memory.\n{}", e)
        }
    }
}

impl MemoryMap {
    fn hdma(&mut self) {
        if matches!(*self.mode, GbMode::DMG) {return;}
        let length = (self.hdma_length as u16 + 1) * 0x10;
        let destination = self.hdma_destination + 0x8000;
        for i in 0..length {
            let val = self.read_u8(self.hdma_source + i, false);
            self.write_u8(val, destination + i, false);
        }
        self.hdma_length = 0xFF;
    }
}

pub trait MemoryMapped {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }
}