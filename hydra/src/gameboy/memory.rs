mod attributes;
mod mbc;
mod oam;
pub mod rom;
mod sram;
pub mod vram;
pub mod wram;

use crate::{
    common::errors::HydraIOError,
    gameboy::{
        InterruptEnable, InterruptFlags, Joypad, memory::{oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{colormap::ColorMap, lcdc::LcdController, state::PpuState}, timer::MasterTimer
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc};

pub struct MemoryMap {
    cartridge: Option<Box<dyn mbc::MemoryBankController>>, // ROM, SRAM
    vram: Rc<RefCell<Vram>>,
    wram: Rc<RefCell<Wram>>,
    oam: Oam,
    hram: [u8; 0x7F],

    joypad: Rc<RefCell<Joypad>>,
    timer: Rc<RefCell<MasterTimer>>,
    interrupt_flags: Rc<RefCell<InterruptFlags>>,
    lcd_controller: Rc<RefCell<LcdController>>,
    ppu_state: Rc<RefCell<PpuState>>,
    scy: Rc<Cell<u8>>,
    scx: Rc<Cell<u8>>,
    color_map: Rc<RefCell<ColorMap>>,
    wy: Rc<Cell<u8>>,
    wx: Rc<Cell<u8>>,
    interrupt_enable: Rc<RefCell<InterruptEnable>>,

    data_bus: Cell<u8>,
}

impl MemoryMap {
    pub fn new(rom: Rom, vram: Rc<RefCell<Vram>>, wram: Rc<RefCell<Wram>>, joypad: Rc<RefCell<Joypad>>, timer: Rc<RefCell<MasterTimer>>, interrupt_flags: Rc<RefCell<InterruptFlags>>, lcd_controller: Rc<RefCell<LcdController>>, ppu_state: Rc<RefCell<PpuState>>, scy: Rc<Cell<u8>>, scx: Rc<Cell<u8>>, color_map: Rc<RefCell<ColorMap>>, wy: Rc<Cell<u8>>, wx: Rc<Cell<u8>>, interrupt_enable: Rc<RefCell<InterruptEnable>>) -> Result<MemoryMap, HydraIOError> {
        Ok(MemoryMap {
            cartridge: Some(rom.into_mbc()?),
            vram,
            wram,
            oam: Oam::new(),
            hram: [0; 0x7F],

            joypad,
            timer,
            interrupt_flags,
            lcd_controller,
            ppu_state,
            scy,
            scx,
            color_map,
            wy,
            wx,
            interrupt_enable,

            data_bus: Cell::new(0),
        })
    }

    pub fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom = fs::read(path)?;
        self.cartridge = Some(Rom::from_vec(rom)?.into_mbc()?);
        Ok(())
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        let read_result = match address {
            0x0000..=0x7FFF => self.cartridge.as_ref().map(|this| this.read_rom_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.borrow().read_u8(address),
            0xA000..=0xBFFF => self.cartridge.as_ref().map(|this| this.read_ram_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.borrow().read_u8(address)),
            0xE000..=0xFDFF => Ok(self.wram.borrow().read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.read(address - oam::ADDRESS_OFFSET),
            0xFF00 => Ok(<Joypad as MemoryMappedIo<0xFF00>>::read(&*self.joypad.borrow())),
            0xFF04 => Ok(<MasterTimer as MemoryMappedIo<0xFF04>>::read(&*self.timer.borrow())),
            0xFF05 => Ok(<MasterTimer as MemoryMappedIo<0xFF05>>::read(&*self.timer.borrow())),
            0xFF06 => Ok(<MasterTimer as MemoryMappedIo<0xFF06>>::read(&*self.timer.borrow())),
            0xFF07 => Ok(<MasterTimer as MemoryMappedIo<0xFF07>>::read(&*self.timer.borrow())),
            0xFF0F => Ok(<InterruptFlags as MemoryMappedIo<0xFF0F>>::read(&*self.interrupt_flags.borrow())),
            0xFF40 => Ok(<LcdController as MemoryMappedIo<0xFF40>>::read(&*self.lcd_controller.borrow())),
            0xFF41 => Ok(<PpuState as MemoryMappedIo<0xFF41>>::read(&*self.ppu_state.borrow())),
            0xFF42 => Ok(self.scy.get()),
            0xFF43 => Ok(self.scx.get()),
            0xFF44 => Ok(<PpuState as MemoryMappedIo<0xFF44>>::read(&*self.ppu_state.borrow())),
            0xFF45 => Ok(<PpuState as MemoryMappedIo<0xFF45>>::read(&*self.ppu_state.borrow())),
            0xFF47 => Ok(<ColorMap as MemoryMappedIo<0xFF47>>::read(&*self.color_map.borrow())),
            0xFF48 => Ok(<ColorMap as MemoryMappedIo<0xFF48>>::read(&*self.color_map.borrow())),
            0xFF49 => Ok(<ColorMap as MemoryMappedIo<0xFF49>>::read(&*self.color_map.borrow())),
            0xFF4A => Ok(self.wy.get()),
            0xFF4B => Ok(self.wx.get()),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => Ok(<InterruptEnable as MemoryMappedIo<0xFFFF>>::read(&*self.interrupt_enable.borrow())),
            _ => Err(HydraIOError::OpenBusAccess)
        };
        match read_result {
            Ok(value) => self.data_bus.set(value),
            Err(HydraIOError::OpenBusAccess) => {}//println!("Warning: Read from open bus at address {:#06X}", address),
            Err(e) => panic!("Error reading from memory.\n{}", e),
        }

        return self.data_bus.get();
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> () {
        self.data_bus.set(value);
        let write_result = match address {
            0x0000..=0x7FFF => self.cartridge.as_mut().map(|this| this.write_rom_u8(value, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.borrow_mut().write_u8(value, address),
            0xA000..=0xBFFF => self.cartridge.as_mut().map(|this| this.write_ram_u8(value, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.borrow_mut().write_u8(value, address)),
            0xE000..=0xFDFF => Ok(self.wram.borrow_mut().write_u8(value, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.write(address - oam::ADDRESS_OFFSET, value),
            0xFF00 => Ok(<Joypad as MemoryMappedIo<0xFF00>>::write(&mut *self.joypad.borrow_mut(), value)),
            0xFF04 => Ok(<MasterTimer as MemoryMappedIo<0xFF04>>::write(&mut *self.timer.borrow_mut(), value)),
            0xFF05 => Ok(<MasterTimer as MemoryMappedIo<0xFF05>>::write(&mut *self.timer.borrow_mut(), value)),
            0xFF06 => Ok(<MasterTimer as MemoryMappedIo<0xFF06>>::write(&mut *self.timer.borrow_mut(), value)),
            0xFF07 => Ok(<MasterTimer as MemoryMappedIo<0xFF07>>::write(&mut *self.timer.borrow_mut(), value)),
            0xFF0F => Ok(<InterruptFlags as MemoryMappedIo<0xFF0F>>::write(&mut *self.interrupt_flags.borrow_mut(), value)),
            0xFF40 => Ok(<LcdController as MemoryMappedIo<0xFF40>>::write(&mut *self.lcd_controller.borrow_mut(), value)),
            0xFF41 => Ok(<PpuState as MemoryMappedIo<0xFF41>>::write(&mut *self.ppu_state.borrow_mut(), value)),
            0xFF42 => Ok(self.scy.set(value)),
            0xFF43 => Ok(self.scx.set(value)),
            0xFF44 => Ok(<PpuState as MemoryMappedIo<0xFF44>>::write(&mut *self.ppu_state.borrow_mut(), value)),
            0xFF45 => Ok(<PpuState as MemoryMappedIo<0xFF45>>::write(&mut *self.ppu_state.borrow_mut(), value)),
            0xFF47 => Ok(<ColorMap as MemoryMappedIo<0xFF47>>::write(&mut *self.color_map.borrow_mut(), value)),
            0xFF48 => Ok(<ColorMap as MemoryMappedIo<0xFF48>>::write(&mut *self.color_map.borrow_mut(), value)),
            0xFF49 => Ok(<ColorMap as MemoryMappedIo<0xFF49>>::write(&mut *self.color_map.borrow_mut(), value)),
            0xFF4A => Ok(self.wy.set(value)),
            0xFF4B => Ok(self.wx.set(value)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            0xFFFF => Ok(<InterruptEnable as MemoryMappedIo<0xFFFF>>::write(&mut *self.interrupt_enable.borrow_mut(), value)),
            _ => Err(HydraIOError::OpenBusAccess)
        };
        match write_result {
            Ok(_) => {}
            Err(HydraIOError::OpenBusAccess) => {}//println!("Warning: Write to open bus at address {:#06X}", address),
            Err(e) => panic!("Error writing to memory.\n{}", e)
        }
    }
}

pub trait MemoryMappedIo<const ADDRESS: u16> {
    fn read(&self) -> u8;
    fn write(&mut self, val: u8);
}

#[repr(u16)]
pub enum MMIO {
    JOYP = 0xFF00,
    SB = 0xFF01,
    SC = 0xFF02,
    DIV = 0xFF04,
    TIMA = 0xFF05,
    TMA = 0xFF06,
    TAC = 0xFF07,
    IF = 0xFF0F,
    NR10 = 0xFF10,
    NR11 = 0xFF11,
    NR12 = 0xFF12,
    NR13 = 0xFF13,
    NR14 = 0xFF14,
    NR21 = 0xFF16,
    NR22 = 0xFF17,
    NR23 = 0xFF18,
    NR24 = 0xFF19,
    NR30 = 0xFF1A,
    NR31 = 0xFF1B,
    NR32 = 0xFF1C,
    NR33 = 0xFF1D,
    NR34 = 0xFF1E,
    NR41 = 0xFF20,
    NR42 = 0xFF21,
    NR43 = 0xFF22,
    NR44 = 0xFF23,
    NR50 = 0xFF24,
    NR51 = 0xFF25,
    NR52 = 0xFF26,
    WAV00 = 0xFF30,
    WAV01 = 0xFF31,
    WAV02 = 0xFF32,
    WAV03 = 0xFF33,
    WAV04 = 0xFF34,
    WAV05 = 0xFF35,
    WAV06 = 0xFF36,
    WAV07 = 0xFF37,
    WAV08 = 0xFF38,
    WAV09 = 0xFF39,
    WAV10 = 0xFF3A,
    WAV11 = 0xFF3B,
    WAV12 = 0xFF3C,
    WAV13 = 0xFF3D,
    WAV14 = 0xFF3E,
    WAV15 = 0xFF3F,
    LCDC = 0xFF40,
    STAT = 0xFF41,
    SCY = 0xFF42,
    SCX = 0xFF43,
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A,
    WX = 0xFF4B,
    KEY0 = 0xFF4C,
    KEY1 = 0xFF4D,
    VBK = 0xFF4F,
    BOOT = 0xFF50,
    HDMA1 = 0xFF51,
    HDMA2 = 0xFF52,
    HDMA3 = 0xFF53,
    HDMA4 = 0xFF54,
    HDMA5 = 0xFF55,
    RP = 0xFF56,
    BCPS = 0xFF68,
    BCPD = 0xFF69,
    OCPS = 0xFF6A,
    OCPD = 0xFF6B,
    OPRI = 0xFF6C,
    SVBK = 0xFF70,
    PCM12 = 0xFF76,
    PCM34 = 0xFF77,
    IE = 0xFFFF
}