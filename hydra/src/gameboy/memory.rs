mod mbc;
pub mod io;
mod oam;
mod sram;
pub mod rom;
pub mod vram;
mod wram;

use crate::{
    common::errors::HydraIOError,
    gameboy::{
        Model,
        memory::{io::IoMap, oam::Oam, rom::Rom, vram::Vram, wram::Wram},
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc};

pub struct MemoryMap {
    cartridge: Option<Box<dyn mbc::MemoryBankController>>, // ROM, SRAM
    vram: Rc<RefCell<Vram>>,
    wram: Box<Wram>,
    oam: Oam,
    io: IoMap,
    hram: [u8; 0x7F],

    data_bus: Cell<u8>,
}

impl MemoryMap {
    pub fn from_rom_and_model(rom: Rom, model: Model, vram: Rc<RefCell<Vram>>, io: IoMap) -> Result<MemoryMap, HydraIOError> {
        Ok(MemoryMap {
            cartridge: Some(rom.into_mbc()?),
            vram,
            wram: Box::new(Wram::new(model, &io)),
            oam: Oam::new(),
            io,
            hram: [0; 0x7F],

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
            0xC000..=0xDFFF => Ok(self.wram.read_u8(address)),
            0xE000..=0xFDFF => Ok(self.wram.read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.read(address - oam::ADDRESS_OFFSET),
            0xFF00..=0xFF7F => self.io.read(address),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => self.io.read(address), // Same as MMIO case, kept separate for clarity
        };
        match read_result {
            Ok(value) => self.data_bus.set(value),
            Err(HydraIOError::OpenBusAccess) => println!("Warning: Read from open bus at address {:#06X}", address),
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
            0xC000..=0xDFFF => Ok(self.wram.write_u8(value, address)),
            0xE000..=0xFDFF => Ok(self.wram.write_u8(value, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.write(address - oam::ADDRESS_OFFSET, value),
            0xFF00..=0xFF7F => self.io.write(value, address),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            0xFFFF => self.io.write(value, address), // Same as MMIO case, kept separate for clarity
        };
        match write_result {
            Ok(_) => {}
            Err(HydraIOError::OpenBusAccess) => println!("Warning: Write to open bus at address {:#06X}", address),
            Err(e) => panic!("Error writing to memory.\n{}", e)
        }
    }
}