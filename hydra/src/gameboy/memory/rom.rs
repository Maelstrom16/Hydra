use std::{ops::RangeInclusive, sync::{Arc, RwLock}};

use crate::{common::{bit::BitVec, errors::HydraIOError}, gameboy::memory::{mbc::{MemoryBankController, huc1::HuC1, huc3::HuC3, mbc0::MBC0, mbc1::MBC1, mbc2::MBC2, mbc3::{MBC3, RealTimeClock}, mbc5::MBC5, mbc6::MBC6, mbc7::MBC7, pocketcamera::PocketCamera, tama5::TAMA5}, sram::Sram}, input::ControllerState};

// Header Registers
pub const TITLE_ADDRESS: RangeInclusive<usize> = 0x0134..=0x0143;
pub const CGB_FLAG_ADDRESS: usize = 0x0143;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0144;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

pub struct RomHeader(Vec<u8>);

impl RomHeader {
    /// Resizes and wraps a byte vector for use as cartridge ROM.
    pub fn from_vec(mut romvec: Vec<u8>) -> Result<Self, HydraIOError> {
        romvec.resize(RomHeader::rom_size_from_vec(&romvec)?, 0x00);
        Ok(RomHeader(romvec))
    }

    /// Consumes this ROM, wrapping it in a new memory bank controller
    pub fn into_mbc(self, controllers: Arc<RwLock<ControllerState>>) -> Result<Box<dyn MemoryBankController>, HydraIOError> {
        match self.0[HARDWARE_ADDRESS] {
            0x00 | 0x08..=0x09 => Ok(Box::new(MBC0::from_header(self)?)),
            0x01..=0x03 => Ok(Box::new(MBC1::from_header(self)?)),
            0x05..=0x06 => Ok(Box::new(MBC2::from_header(self)?)),
            0x0B..=0x0D => panic!("MMM01 not yet supported"),
            0x0F..=0x13 => Ok(Box::new(MBC3::from_header(self)?)),
            0x19..=0x1E => Ok(Box::new(MBC5::from_header(self, controllers)?)),
            0x20 => Ok(Box::new(MBC6::from_header(self)?)),
            0x22 => Ok(Box::new(MBC7::from_header(self, controllers)?)),
            0xFC => Ok(Box::new(PocketCamera::from_header(self)?)),
            0xFD => Ok(Box::new(TAMA5::from_header(self, controllers)?)),
            0xFE => Ok(Box::new(HuC3::from_header(self)?)),
            0xFF => Ok(Box::new(HuC1::from_header(self)?)),
            _ => Err(HydraIOError::MalformedROM("Undefined cartridge hardware identifier").into()),
        }
    }

    // Builds a `Rom` using the `Vec` wrapped by this `RomHeader`
    pub fn into_rom<const BYTES_PER_BANK: usize>(self) -> Rom<BYTES_PER_BANK> {
        let (banked, []) = self.0.as_chunks::<{BYTES_PER_BANK}>() else {panic!("FATAL: ROM resize failed")};
        Rom(Box::from(banked))
    }
    
    /// Reads the hardware identifier from a byte vector, treating it as a ROM cartridge header.
    /// Then, uses the hardware identifier to. determine the number of bytes in one ROM bank.
    fn bank_size_from_vec(romvec: &Vec<u8>) -> Result<usize, HydraIOError> {
        match romvec.get(HARDWARE_ADDRESS) {
            Some(0x20) => Ok(0x2000),
            Some(_) => Ok(0x4000),
            None => Err(HydraIOError::MalformedROM("Undefined ROM size identifier").into())
        }
    }

    /// Reads the ROM size (in bytes) from a byte vector, treating it as a ROM cartridge header.
    fn rom_size_from_vec(romvec: &Vec<u8>) -> Result<usize, HydraIOError> {
        match romvec.get(ROM_SIZE_ADDRESS) {
            Some(0x00) => Ok(0x8000), // 32 KiB
            Some(0x01) => Ok(0x10000), // 64 KiB
            Some(0x02) => Ok(0x20000), // 128 KiB
            Some(0x03) => Ok(0x40000), // 256 KiB
            Some(0x04) => Ok(0x80000), // 512 KiB
            Some(0x05) => Ok(0x100000), // 1 MiB
            Some(0x06) => Ok(0x200000), // 2 MiB
            Some(0x07) => Ok(0x400000), // 4 MiB
            Some(0x08) => Ok(0x800000), // 8 MiB
            Some(0x52) => Ok(0x120000), // 1.1 MiB
            Some(0x53) => Ok(0x140000), // 1.2 MiB
            Some(0x54) => Ok(0x180000), // 1.5 MiB
            _ => Err(HydraIOError::MalformedROM("Undefined ROM size identifier").into()),
        }
    }

    /// Reads the cartridge's RAM size (in banks) from this ROM's header.
    pub fn get_ram_bank_count(&self) -> Result<usize, HydraIOError> {
        match self.0[RAM_SIZE_ADDRESS] {
            0x00 => Ok(0), // 0 KiB
            0x01 => Err(HydraIOError::MalformedROM("2 KiB RAMs are currently unsupported").into()), // 2 KiB?
            0x02 => Ok(1), // 8 KiB
            0x03 => Ok(4), // 32 KiB
            0x04 => Ok(16), // 128 KiB
            0x05 => Ok(8), // 64 KiB
            _ => Err(HydraIOError::MalformedROM("Undefined RAM size identifier").into()),
        }
    }

    /// Reads the cartridge's title from this ROM's header.
    pub fn get_title(&self) -> &[u8] {
        &self.0[TITLE_ADDRESS]
    }

    /// Returns whether this ROM supports CGB registers/features.
    pub fn supports_cgb_mode(&self) -> bool {
        self.0[CGB_FLAG_ADDRESS].test_bit(7)
    }

    /// Constructs a new `RealTimeClock` if the provided ROM indicates a need for one.
    pub fn get_rtc(&self) -> Option<RealTimeClock> {
        match self.0[HARDWARE_ADDRESS] {
            0x0F..=0x10 => Some(RealTimeClock::new()),
            _ => None,
        }
    }

    /// Constructs a rumble status if the provided ROM indicates a need for one.
    pub fn get_rumble(&self) -> Option<bool> {
        match self.0[HARDWARE_ADDRESS] {
            0x1C..=0x1E => Some(false),
            _ => None,
        }
    }

    /// Reads the cartridge's header checksum.
    pub fn get_header_checksum(&self) -> u8 {
        self.0[HEADER_CHECKSUM_ADDRESS]
    }
    
    /// Returns true if the cartridge has a licensee ID of 0x01;
    /// i.e., was published by R&D1.
    pub fn has_publisher_rnd1(&self) -> bool {
        self.0[OLD_LICENSEE_CODE_ADDRESS] == 0x01 || self.0[OLD_LICENSEE_CODE_ADDRESS] == 0x33 && self.0[NEW_LICENSEE_CODE_ADDRESS] == 0x01
    }
}

pub struct Rom<const BYTES_PER_BANK: usize>(Box<[[u8; BYTES_PER_BANK]]>);

impl<const BYTES_PER_BANK: usize> Rom<BYTES_PER_BANK> {
    /// Reads the byte from this ROM at the provided address and bank. 
    pub fn read_bank(&self, address: u16, bank: usize) -> u8 {
        self.0[bank][address as usize]
    }

    /// Returns the number of banks this ROM consists of.
    pub fn get_bank_count(&self) -> usize {
        self.0.len()
    }

    /// Returns the size (in bytes) of this ROM's banks.
    pub const fn bank_size(&self) -> usize {
        BYTES_PER_BANK
    }
}