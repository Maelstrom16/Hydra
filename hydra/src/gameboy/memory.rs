mod mbc;
pub mod oam;
pub mod rom;
mod sram;
pub mod vram;
pub mod wram;

use crate::{
    common::{errors::HydraIOError, util::Accessors},
    gameboy::{
        GbMode, Model, apu::{Apu, channel::{Noise, Pulse, PulseType, Wave}, state::ApuState}, interrupt::{InterruptEnable, InterruptFlags}, joypad::Joypad, memory::{oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{PpuMode, colormap::{self, ColorMap}, lcdc::LcdController, state::PpuState}, serial::SerialConnection, timer::MasterTimer
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc, time::Duration};

pub struct MemoryMap {
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
    pub(super) lcd_controller: LcdController,
    pub(super) ppu_state: PpuState,
    pub(super) scy: u8,
    pub(super) scx: u8,
    pub(super) color_map: Box<dyn ColorMap>,
    pub(super) wy: u8,
    pub(super) wx: u8,

    dma_source: u8,
    dma_cycle: Option<u8>,

    hdma_source: u16,
    hdma_destination: u16,
    hdma_length: u8,

    hram: [u8; 0x7F],
    pub(super) interrupt_enable: InterruptEnable,
}

impl MemoryMap {
    pub fn new(model: &Rc<Model>, mode: Rc<GbMode>) -> Result<MemoryMap, HydraIOError> {
        let interrupt_flags = InterruptFlags::new();
        let interrupt_enable = InterruptEnable::new();
        let joypad = Joypad::new();
        let lcd_controller = LcdController::new();
        let vram = Vram::new(model.clone());
        let wram = Wram::new(model.clone());
        let ppu_state = PpuState::new(&model);
        let timer = MasterTimer::new(model.clone());
        let scy = 0x00;
        let scx = 0x00;
        let wy = 0x00;
        let wx = 0x00;
        let color_map = colormap::from_mode(&mode);
        let oam = Oam::new(model.clone());
        let apu_state = ApuState::new();

        Ok(MemoryMap {
            mode,

            cartridge: None,
            vram,
            wram,
            oam,

            joypad,
            serial: SerialConnection::new(model.clone()),
            timer,
            interrupt_flags,
            apu_state,
            lcd_controller,
            ppu_state,
            scy,
            scx,
            color_map,
            wy,
            wx,

            dma_source: match model.is_monochrome() {
                true => 0xFF,
                false => 0x00,
            },
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

    fn is_vram_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3 when LCD is enabled
        self.ppu_state.get_mode() != PpuMode::Render || !self.lcd_controller.is_lcd_enabled()
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
        let mem_accessible = is_dma || matches!(self.dma_cycle, None);
        let read_result = if mem_accessible { match address {
            0x0000..=0x7FFF => self.cartridge.as_ref().map(|this| this.read_rom_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.read_u8(address),
            0xA000..=0xBFFF => self.cartridge.as_ref().map(|this| this.read_ram_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.read_u8(address)),
            0xE000..=0xFDFF => Ok(self.wram.read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.read(address),
            0xFF00 => Ok(self.joypad.read_joyp()),
            0xFF01..=0xFF02 => self.serial.read(address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.read(address),
            0xFF0F => self.interrupt_flags.read(address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F => self.apu_state.read(address),
            0xFF40 => Ok(self.lcd_controller.read_lcdc()),
            0xFF41 => Ok(self.ppu_state.read_stat()),
            0xFF42 => Ok(self.scy),
            0xFF43 => Ok(self.scx),
            0xFF44 => Ok(self.ppu_state.read_ly()),
            0xFF45 => Ok(self.ppu_state.read_lyc()),
            0xFF46 => Ok(self.dma_source),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.read(address),
            0xFF4A => Ok(self.wy),
            0xFF4B => Ok(self.wx),
            0xFF4F => Ok(self.vram.read_vbk()),
            0xFF51 => Ok(0xFF),
            0xFF52 => Ok(0xFF),
            0xFF53 => Ok(0xFF),
            0xFF54 => Ok(0xFF),
            0xFF55 => Ok(self.hdma_length),
            0xFF70 => Ok(self.wram.read_wbk()),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => self.interrupt_enable.read(address),
            _ => Err(HydraIOError::OpenBusAccess)
        }} else { match address {
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            _ => Err(HydraIOError::OpenBusAccess)
        }};

        match read_result {
            Ok(value) => value,
            Err(HydraIOError::OpenBusAccess) => 0xFF, //println!("Warning: Read from open bus at address {:#06X}", address),
            Err(e) => panic!("Error reading from memory.\n{}", e),
        }
    }

    pub fn write_u8(&mut self, val: u8, address: u16, is_dma: bool) -> () {
        let mem_accessible = is_dma || matches!(self.dma_cycle, None);
        let write_result = if mem_accessible { match address {
            0x0000..=0x7FFF => self.cartridge.as_mut().map(|this| this.write_rom_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.write_u8(val, address),
            0xA000..=0xBFFF => self.cartridge.as_mut().map(|this| this.write_ram_u8(val, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.write_u8(val, address)),
            0xE000..=0xFDFF => Ok(self.wram.write_u8(val, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.write(address, val),
            0xFF00 => Ok(self.joypad.write_joyp(val, &mut self.interrupt_flags)),
            0xFF01..=0xFF02 => self.serial.write(val, address),
            0xFF04..=0xFF07 | 0xFF4D => self.timer.write(val, address, &mut self.apu_state),
            0xFF0F => self.interrupt_flags.write(val, address),
            0xFF10..=0xFF14 | 0xFF16..=0xFF1E | 0xFF20..=0xFF26 | 0xFF30..=0xFF3F => self.apu_state.write(val, address),
            0xFF40 => Ok(self.lcd_controller.write_lcdc(val)),
            0xFF41 => Ok(self.ppu_state.write_stat(val)),
            0xFF42 => Ok(self.scy = val),
            0xFF43 => Ok(self.scx = val),
            0xFF44 => Ok(self.ppu_state.write_ly(val)),
            0xFF45 => Ok(self.ppu_state.write_lyc(val, &mut self.interrupt_flags)),
            0xFF46 => Ok({self.dma_source = val; self.dma_cycle = Some(0);}),
            0xFF47..=0xFF49 | 0xFF68..=0xFF6B => self.color_map.write(val, address),
            0xFF4A => Ok(self.wy = val),
            0xFF4B => Ok(self.wx = val),
            0xFF4F => Ok(self.vram.write_vbk(val)),
            0xFF51 => Ok(self.hdma_source = (self.hdma_source & 0xFF) | ((val as u16) << 8)),
            0xFF52 => Ok(self.hdma_source = (self.hdma_source & 0xFF00) | (val as u16)),
            0xFF53 => Ok(self.hdma_destination = (self.hdma_destination & 0xFF) | ((val as u16 & 0x1F) << 8)),
            0xFF54 => Ok(self.hdma_destination = (self.hdma_destination & 0xFF00) | (val as u16)),
            0xFF55 => Ok({self.hdma_length = val; self.hdma();}),
            0xFF70 => Ok(self.wram.write_wbk(val)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = val),
            0xFFFF => self.interrupt_enable.write(val, address),
            _ => Err(HydraIOError::OpenBusAccess)
        }} else { match address {
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = val),
            _ => Err(HydraIOError::OpenBusAccess)
        }};
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