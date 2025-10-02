mod cartmbc;
mod consmbc;
use crate::{common::errors::HydraIOError, gameboy::Model};
use std::fs;

pub const TITLE_ADDRESS: usize = 0x0134;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0147;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

pub struct Memory {
    cartridge: Option<Box<dyn cartmbc::CartMemoryBankController>>,
    console: Box<dyn consmbc::ConsMemoryBankController>,
    data_bus: u8,
}

impl Memory {
    pub fn from_model(model: Model) -> Result<Memory, HydraIOError> {
        let result_cart = Memory {
            cartridge: Option::None,
            console: consmbc::from_model(model)?,
            data_bus: 0,
        };
        Ok(result_cart)
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), HydraIOError> {
        let rom = fs::read(path)?;
        self.cartridge = Some(cartmbc::from_rom(&rom)?);
        Ok(())
    }

    pub fn read_u8(&mut self, address: u16) -> u8 {
        let read_result = match address {
            0x0000..0x8000 => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_rom_u8(address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..0xA000 => self.console.read_vram_u8((address - 0x8000) as usize),
            0xA000..0xC000 => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_ram_u8((address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..0xE000 => self.console.read_wram_u8((address - 0xC000) as usize),
            0xE000..0xFE00 => self.console.read_wram_u8((address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00.. => panic!("OAM / IO / HRAM not yet implemented"),
        };
        match read_result {
            Ok(value) => self.data_bus = value,
            Err(e) => match e {
                HydraIOError::OpenBusAccess => {}
                _ => panic!("Error reading from memory.\n{}", e),
            },
        }

        return self.data_bus;
    }

    pub fn read_i8(&mut self, address: u16) -> i8 {
        return self.read_u8(address) as i8;
    }

    pub fn read_u16(&mut self, address: u16) -> u16 {
        return u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)]);
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> () {
        self.data_bus = value;
        let write_result = match address {
            0x0000..0x8000 => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_rom_u8(value, address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..0xA000 => self
                .console
                .write_vram_u8(value, (address - 0x8000) as usize),
            0xA000..0xC000 => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_ram_u8(value, (address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..0xE000 => self
                .console
                .write_wram_u8(value, (address - 0xC000) as usize),
            0xE000..0xFE00 => self
                .console
                .write_wram_u8(value, (address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00.. => panic!("OAM / IO / HRAM not yet implemented"),
        };
        if let Err(e) = write_result {
            panic!("Error writing to memory.\n{}", e);
        }
    }

    pub fn write_u16(&mut self, value: u16, address: u16) -> () {
        let bytes: [u8; 2] = u16::to_le_bytes(value);
        self.write_u8(bytes[0], address);
        self.write_u8(bytes[1], address + 1);
    }
}
