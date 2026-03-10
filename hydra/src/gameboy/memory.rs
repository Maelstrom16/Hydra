pub mod hdma;
mod mbc;
pub mod oam;
pub mod rom;
mod sram;
pub mod vram;
pub mod wram;

use crate::{
    common::errors::HydraIOError, deserialize, gameboy::{
        GbMode, Model, apu::{Apu, channel::{Noise, Pulse, PulseType, Wave}, state::ApuState}, interrupt::{InterruptEnable, InterruptFlags}, joypad::Joypad, memory::{hdma::HdmAccessor, oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{PpuMode, colormap::{self, ColorMap}, state::PpuState}, serial::SerialConnection, timer::MasterTimer
    }, serialize
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
    pub(super) hdma: Box<dyn HdmAccessor>,

    dma_source: u8,
    dma_cycle: Option<u8>,

    hram: [u8; 0x7F],
    cgb_misc: [u8; 4],
    pub(super) interrupt_enable: InterruptEnable,
}

impl MemoryMap {
    pub fn new(model: Rc<Model>, mode: Rc<GbMode>) -> Result<MemoryMap, HydraIOError> {
        let interrupt_flags = InterruptFlags::new();
        let interrupt_enable = InterruptEnable::new();
        let joypad = Joypad::new(&model);
        let serial = SerialConnection::new(mode.clone());
        let vram = Vram::new(model.clone(), mode.clone());
        let wram = Wram::new(mode.clone());
        let ppu_state = PpuState::new(&model);
        let timer = MasterTimer::new(model.clone(), mode.clone());
        let color_map = colormap::from_mode(&mode);
        let oam = Oam::new(mode.clone());
        let apu_state = ApuState::new(model.clone());
        let dma_source = match model.is_monochrome() {
            true => 0xFF,
            false => 0x00,
        };
        let hdma = hdma::from_mode(&mode);

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
            hdma,

            dma_source,
            dma_cycle: None,

            hram: [0; 0x7F],
            cgb_misc: [0; 4],
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

    fn is_oam_accessible(&self) -> bool {
        // Only when LCD is disabled, or when outside of OAM DMA during H/Vblank
        !self.ppu_state.is_lcd_enabled() || (matches!(self.dma_cycle, None) && matches!(self.ppu_state.get_mode(), PpuMode::HBlank | PpuMode::VBlank))
    }

    pub fn tick_dma(&mut self) {
        if let Some(cycle) = self.dma_cycle {
            let source_address = (self.dma_source as u16) << 8 | cycle as u16;
            let destination_address = 0xFE00 | cycle as u16;
            self.oam.write(destination_address, self.read_u8(source_address, true));

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
            0xE000..=0xFFFF if is_dma => Ok(self.wram.read_u8(address - 0x2000)), // OAM DMA echo RAM mirrors the entirety of WRAM
            0xE000..=0xFDFF if self.is_wram_accessible() => Ok(self.wram.read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF if self.is_oam_accessible() => self.oam.read(address),
            0xFF00 => Ok(self.joypad.read_joyp()),
            0xFF01..=0xFF02 => self.serial.read(address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.read(address),
            0xFF0F => self.interrupt_flags.read(address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F | 0xFF76..=0xFF77 => self.apu_state.read(address),
            0xFF40..=0xFF45 | 0xFF4A..=0xFF4B => self.ppu_state.read(address),
            0xFF46 => Ok(self.dma_source),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.read(address),
            0xFF4F => self.vram.read_vbk(),
            0xFF51..=0xFF55 => self.hdma.read(address),
            0xFF70 if matches!(*self.mode, GbMode::CGB) => Ok(self.wram.read_wbk()),
            0xFF72..=0xFF73 if self.model.is_color() => Ok(self.cgb_misc[address as usize - 0xFF72]),
            0xFF74 if matches!(*self.mode, GbMode::CGB) => Ok(self.cgb_misc[2]),
            0xFF75 if self.model.is_color() => Ok(self.cgb_misc[3] | 0b10001111),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => self.interrupt_enable.read(address),
            _ => Err(HydraIOError::OpenBusAccess)
        };

        match read_result {
            Ok(value) => value,
            Err(HydraIOError::OpenBusAccess) => {
                println!("Warning: Read from open bus at address {:#06X}", address);
                0xFF
            }
            Err(e) => panic!("Error reading from memory.\n{}", e),
        }
    }

    pub fn write_u8(&mut self, val: u8, address: u16) -> () {
        let write_result = match address {
            0x0000..=0x7FFF if self.is_cart_accessible() => self.cartridge.as_mut().map(|this| this.write_rom_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF if self.is_vram_accessible() => self.vram.write_u8(val, address),
            0xA000..=0xBFFF if self.is_cart_accessible() => self.cartridge.as_mut().map(|this| this.write_ram_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF if self.is_wram_accessible() => Ok(self.wram.write_u8(val, address)),
            0xE000..=0xFDFF if self.is_wram_accessible() => Ok(self.wram.write_u8(val, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF if self.is_oam_accessible() => self.oam.write(address, val),
            0xFF00 => Ok(self.joypad.write_joyp(val, &mut self.interrupt_flags)),
            0xFF01..=0xFF02 => self.serial.write(val, address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.write(val, address, &mut self.apu_state),
            0xFF0F => self.interrupt_flags.write(val, address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F | 0xFF76..=0xFF77 => self.apu_state.write(val, address),
            0xFF40..=0xFF45 | 0xFF4A..=0xFF4B => self.ppu_state.write(val, address, &mut self.interrupt_flags),
            0xFF46 => Ok({self.dma_source = val; self.dma_cycle = Some(0);}),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.write(val, address),
            0xFF4F => self.vram.write_vbk(val),
            0xFF51..=0xFF55 => self.hdma.write(val, address, &self.ppu_state),
            0xFF70 if matches!(*self.mode, GbMode::CGB) => Ok(self.wram.write_wbk(val)),
            0xFF72..=0xFF73 if self.model.is_color() => Ok(self.cgb_misc[address as usize - 0xFF72] = val),
            0xFF74 if matches!(*self.mode, GbMode::CGB) => Ok(self.cgb_misc[2] = val),
            0xFF75 if self.model.is_color() => Ok(self.cgb_misc[3] = val | 0b10001111),
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

pub trait MemoryMapped {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }
}