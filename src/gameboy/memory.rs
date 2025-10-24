mod cartmbc;
mod consmbc;
pub mod io;
mod oam;

use crate::{common::errors::HydraIOError, gameboy::{cpu::CPU, memory::{io::IO, oam::OAM}, ppu::PPU, Model}};
use std::{cell::{Cell, RefCell}, fs, ops::Index, rc::Rc, sync::RwLock};

// Header Registers
pub const TITLE_ADDRESS: usize = 0x0134;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0144;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

pub struct Memory {
    cartridge: Option<Box<dyn cartmbc::CartMemoryBankController>>, // ROM, SRAM
    console: Box<dyn consmbc::ConsMemoryBankController>, // VRAM, WRAM/Echo RAM
    hram: [u8; 0x7F],
    io: IO,
    oam: OAM,

    data_bus: Cell<u8>,
}

impl Memory {
    pub fn from_rom_and_model(rom: Box<[u8]>, model: Model, io: IO) -> Result<Memory, HydraIOError> {
        let result_cart = Memory {
            cartridge: Some(cartmbc::from_rom(rom)?),
            console: consmbc::from_model(model),
            hram: [0; 0x7F],
            io,
            oam: OAM::new(),
            data_bus: Cell::new(0),
        };
        Ok(result_cart)
    }

    pub fn hot_swap_rom(&mut self, rom: Box<[u8]>) -> Result<(), HydraIOError> {
        self.cartridge = Some(cartmbc::from_rom(rom)?);
        Ok(())
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        let read_result = match address {
            0x0000..=0x7FFF => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_rom_u8(address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..=0x9FFF => self.console.read_vram_u8((address - 0x8000) as usize),
            0xA000..=0xBFFF => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_ram_u8((address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..=0xDFFF => self.console.read_wram_u8((address - 0xC000) as usize),
            0xE000..=0xFDFF => self.console.read_wram_u8((address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00..=0xFEFF => self.oam.read(address as usize - oam::ADDRESS_OFFSET),
            0xFF00..=0xFF7F => Ok(self.io[address as usize - io::ADDRESS_OFFSET].get()),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => Ok(self.io[io::IE].get()),
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
            0x0000..=0x7FFF => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_rom_u8(value, address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..=0x9FFF => self.console.write_vram_u8(value, (address - 0x8000) as usize),
            0xA000..=0xBFFF => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_ram_u8(value, (address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..=0xDFFF => self.console.write_wram_u8(value, (address - 0xC000) as usize),
            0xE000..=0xFDFF => self.console.write_wram_u8(value, (address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00..=0xFEFF => self.oam.write(address as usize - oam::ADDRESS_OFFSET, value),
            0xFF00..=0xFF7F => Ok(self.io[address as usize - io::ADDRESS_OFFSET].set(value)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            0xFFFF => Ok(self.io[io::IE].set(value)),
        };
        if let Err(e) = write_result {
            panic!("Error writing to memory.\n{}", e);
        }
    }
}
