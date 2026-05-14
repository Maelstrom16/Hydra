use std::sync::{Arc, RwLock};

use crate::common::bit::BitVec;
use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};
use crate::input::{ControllerMessage, ControllerState};

pub struct TAMA5 {
    rom: Rom<0x4000>,
    ram: [u8; 32],
    controllers: Arc<RwLock<ControllerState>>,

    rom_bank: u8,
    ram_bank: u8,

    ram_area_mode: u8,
    ram_area_addr: u8,
    ram_area_data: u8,
}

impl TAMA5 {
    pub fn from_header(header: RomHeader, controllers: Arc<RwLock<ControllerState>>) -> Result<Self, HydraIOError> {
        Ok(TAMA5 {
            rom: header.into_rom(),
            ram: [0x00; 32],
            controllers,

            rom_bank: 1,
            ram_bank: 0,

            ram_area_mode: 0x00,
            ram_area_addr: 0b00000,
            ram_area_data: 0x00,
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for TAMA5 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        println!("RAM READ {:#06X}", address);
        match address {
            0xA000..=0xBFFF if address.test_bit(0) => Err(HydraIOError::OpenBusAccess),
            0xA000..=0xBFFF /* if !address.test_bit(0) */ => match (self.ram_area_mode, self.ram_area_addr & 0b11100000) {
                (0x0A, _) => Ok(0xF1),
                (0x0C, _) => Ok(self.ram_area_data | 0xF0),
                (0x0D, _) => Ok((self.ram_area_data >> 4) | 0xF0),
                _ => Err(HydraIOError::OpenBusAccess),
            }
            _ => unimplemented!("Attempted to read from invalid SRAM address {:#06X}", address)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        println!("ROM WRITE {:#04X}, {:#06X}", value, address);
        Ok(match address {
            0x0000..=0x7FFF => { /* Do nothing */ }
            _ => unimplemented!("Attempted to write {:#04X} to invalid ROM address {:#06X}", value, address)
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        println!("RAM WRITE {:#04X}, {:#06X}", value, address);
        Ok(match address {
            0xA000..=0xBFFF if address.test_bit(0) => { self.ram_area_mode = value } 
            0xA000..=0xBFFF /* if !address.test_bit(0) */ => match self.ram_area_mode {
                0x00 => { self.rom_bank = (self.rom_bank & 0b00010000) | (value & 0b1111) }
                0x01 => { self.rom_bank = (self.rom_bank & 0b00001111) | (value & 0b0001) << 4 }
                0x04 => { self.ram_area_data = (self.ram_area_data & 0b11110000) | (value & 0b1111) }
                0x05 => { self.ram_area_data = (self.ram_area_data & 0b00001111) | (value & 0b1111) << 4 }
                0x06 => { self.ram_area_addr = (self.ram_area_addr & 0b00001111) | (value & 0b1111) << 4 }
                0x07 => { 
                    self.ram_area_addr = (self.ram_area_addr & 0b11110000) | (value & 0b1111);
                    let sram_address = (self.ram_area_addr & 0b11111) as usize;
                    match self.ram_area_addr {
                        0x00..=0x1F => { self.ram[sram_address] = value }
                        0x20..=0x3F => { self.ram_area_data = self.ram[sram_address] }
                        0x70..=0x7F => { self.ram_area_data = self.ram_area_addr }
                        _ => {}
                    }
                }
                _ => {}
            }
            _ => unimplemented!("Attempted to write {:#04X} to invalid SRAM address {:#06X}", value, address)
        })
    }
}
