use std::ops::RangeInclusive;

use crate::{common::errors::HydraIOError, gameboy::memory::{mbc::{MemoryBankController, mbc0::MBC0, mbc1::MBC1, mbc2::MBC2, mbc3::{MBC3, RealTimeClock}, mbc5::MBC5}, sram::Sram}};

// Header Registers
pub const TITLE_ADDRESS: RangeInclusive<usize> = 0x0134..=0x0143;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0144;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

pub struct Rom(Box<[[u8; Rom::BYTES_PER_BANK]]>);

impl Rom {
    /// The number of bytes in one bank of cartridge ROM.
    pub const BYTES_PER_BANK: usize = 0x4000;

    /// Resizes and wraps a byte vector for use as cartridge ROM.
    pub fn from_vec(mut romvec: Vec<u8>) -> Result<Self, HydraIOError> {
        romvec.resize(Rom::rom_size_from_vec(&romvec)?, 0x00);
        let (banked, []) = romvec.as_chunks() else {panic!("FATAL: ROM resize failed")};
        
        Ok(Rom(Box::from(banked)))
    }

    /// Consumes this ROM, wrapping it in a new memory bank controller
    pub fn into_mbc(self) -> Result<Box<dyn MemoryBankController>, HydraIOError> {
        match self.0[0][HARDWARE_ADDRESS] {
            0x00 | 0x08..=0x09 => Ok(Box::new(MBC0::from_rom(self)?)),
            0x01..=0x03 => Ok(Box::new(MBC1::from_rom(self)?)),
            0x05..=0x06 => Ok(Box::new(MBC2::from_rom(self)?)),
            0x0B..=0x0D => panic!("MMM01 not yet supported"),
            0x0F..=0x13 => Ok(Box::new(MBC3::from_rom(self)?)),
            0x19..=0x1E => Ok(Box::new(MBC5::from_rom(self)?)),
            0x20 => panic!("MBC6 not yet supported"),
            0x22 => panic!("MBC7 not yet supported"),
            0xFC => panic!("POCKET CAMERA not yet supported"),
            0xFD => panic!("TAMA5 not yet supported"),
            0xFE => panic!("HuC3 not yet supported"),
            0xFF => panic!("HuC1 not yet supported"),
            _ => Err(HydraIOError::MalformedROM("Undefined cartridge hardware identifier").into()),
        }
    }    

    /// Reads the byte from this ROM at the provided address and bank. 
    pub fn read_bank(&self, address: u16, bank: usize) -> u8 {
        self.0[bank][address as usize]
    }

    /// Reads the cartridge's title from this ROM's header.
    pub fn get_title(&self) -> &[u8] {
        &self.0[0][TITLE_ADDRESS]
    }

    /// Constructs a new `RealTimeClock` if the provided ROM indicates a need for one.
    pub fn get_rtc(&self) -> Option<RealTimeClock> {
        match self.0[0][HARDWARE_ADDRESS] {
            0x0F..=0x10 => Some(RealTimeClock::new()),
            _ => None,
        }
    }

    /// Reads the ROM size (in bytes) from a byte vector, treating it as a ROM cartridge header.
    fn rom_size_from_vec(romvec: &Vec<u8>) -> Result<usize, HydraIOError> {
        let bank_count: Result<usize, HydraIOError> = match romvec.get(ROM_SIZE_ADDRESS) {
            Some(0x00) => Ok(2), // 32 KiB
            Some(0x01) => Ok(4), // 64 KiB
            Some(0x02) => Ok(8), // 128 KiB
            Some(0x03) => Ok(16), // 256 KiB
            Some(0x04) => Ok(32), // 512 KiB
            Some(0x05) => Ok(64), // 1 MiB
            Some(0x06) => Ok(128), // 2 MiB
            Some(0x07) => Ok(256), // 4 MiB
            Some(0x08) => Ok(512), // 8 MiB
            Some(0x52) => Ok(72), // 1.1 MiB
            Some(0x53) => Ok(80), // 1.2 MiB
            Some(0x54) => Ok(96), // 1.5 MiB
            _ => Err(HydraIOError::MalformedROM("Undefined ROM size identifier").into()),
        };

        Ok(bank_count? * Rom::BYTES_PER_BANK)
    }

    /// Returns the number of banks this ROM consists of.
    pub fn get_bank_count(&self) -> usize {
        self.0.len()
    }

    /// Reads the cartridge's RAM size (in banks) from this ROM's header.
    pub fn get_ram_bank_count(&self) -> Result<usize, HydraIOError> {
        match self.0[0][RAM_SIZE_ADDRESS] {
            0x00 => Ok(0), // 0 KiB
            0x01 => Err(HydraIOError::MalformedROM("2 KiB RAMs are currently unsupported").into()), // 2 KiB?
            0x02 => Ok(1), // 8 KiB
            0x03 => Ok(4), // 32 KiB
            0x04 => Ok(16), // 128 KiB
            0x05 => Ok(8), // 64 KiB
            _ => Err(HydraIOError::MalformedROM("Undefined RAM size identifier").into()),
        }
    }
    /// Reads the cartridge's RAM size (in bytes) from this ROM's header.
    pub fn get_ram_size(&self) -> Result<usize, HydraIOError> {
        Ok(self.get_ram_bank_count()? * Sram::BYTES_PER_BANK)
    }

    /// Reads the cartridge's header checksum.
    pub fn get_header_checksum(&self) -> u8 {
        self.0[0][HEADER_CHECKSUM_ADDRESS]
    }
}

impl Rom {
    /// Returns true if the cartridge has a licensee ID of 0x01;
    /// i.e., was published by R&D1.
    pub fn has_publisher_rnd1(&self) -> bool {
        self.0[0][OLD_LICENSEE_CODE_ADDRESS] == 0x01 || self.0[0][OLD_LICENSEE_CODE_ADDRESS] == 0x33 && self.0[0][NEW_LICENSEE_CODE_ADDRESS] == 0x01
    }
}