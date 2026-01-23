mod mbc;
pub mod io;
mod oam;
pub mod vram;
mod wram;

use crate::{
    common::errors::HydraIOError,
    gameboy::{
        Model,
        memory::{io::IOMap, oam::OAM, vram::Vram, wram::Wram},
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc};

// Header Registers
pub const TITLE_ADDRESS: usize = 0x0134;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0144;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

pub struct Memory {
    cartridge: Option<Box<dyn mbc::MemoryBankController>>, // ROM, SRAM
    vram: Rc<RefCell<Vram>>,
    wram: Box<Wram>,
    oam: OAM,
    io: IOMap,
    hram: [u8; 0x7F],

    data_bus: Cell<u8>,
}

impl Memory {
    pub fn from_rom_and_model(rom: Box<[u8]>, model: Model, io: IOMap) -> Result<Memory, HydraIOError> {
        let result_cart = Memory {
            cartridge: Some(mbc::from_rom(rom)?),
            vram: Rc::new(RefCell::new(Vram::new(model, &io))),
            wram: Box::new(Wram::new(model, &io)),
            oam: OAM::new(),
            io,
            hram: [0; 0x7F],

            data_bus: Cell::new(0),
        };
        Ok(result_cart)
    }

    pub fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom: Box<[u8]> = fs::read(path)?.into_boxed_slice();
        self.cartridge = Some(mbc::from_rom(rom)?);
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
            0xFF00..=0xFF7F => Ok(self.io.read(address)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => Ok(self.io.read(address)), // Same as MMIO case, kept separate for clarity
        };
        match read_result {
            Ok(value) => self.data_bus.set(value),
            Err(e) => match e {
                HydraIOError::OpenBusAccess => {}
                _ => panic!("Error reading from memory.\n{}", e),
            },
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
            0xFF00..=0xFF7F => Ok(self.io.write(value, address)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            0xFFFF => Ok(self.io.write(value, address)), // Same as MMIO case, kept separate for clarity
        };
        if let Err(e) = write_result {
            panic!("Error writing to memory.\n{}", e);
        }
    }

    pub fn get_io(&self) -> &IOMap {
        return &self.io;
    }

    pub fn get_vram(&self) -> Rc<RefCell<Vram>> {
        return self.vram.clone();
    }
}