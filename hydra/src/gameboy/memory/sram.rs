use crate::{common::errors::HydraIOError, gameboy::memory::rom::{Rom, RomHeader}};

pub const ADDRESS_OFFSET: usize = 0xA000;

pub struct Sram<const BYTES_PER_BANK: usize>(Box<[[u8; BYTES_PER_BANK]]>);

impl<const BYTES_PER_BANK: usize> Sram<BYTES_PER_BANK> {
    /// Constructs a new SRAM using the size specified by a cartridge ROM.
    pub fn from_header(header: &RomHeader) -> Result<Self, HydraIOError> {
        Ok(Sram(vec![[0x00; BYTES_PER_BANK]; header.get_ram_bank_count()?].into_boxed_slice()))
    }

    /// Reads the byte from this SRAM at the provided address and bank. 
    pub fn read_bank(&self, address: u16, bank: usize) -> u8 {
        self.0[bank][address as usize]
    }

    /// Writes a value to the byte at the provided address and bank in this SRAM. 
    pub fn write_bank(&mut self, value: u8, address: u16, bank: usize) {
        self.0[bank][address as usize] = value
    }

    /// Returns the number of banks this SRAM consists of.
    pub fn get_bank_count(&self) -> usize {
        self.0.len()
    }

    /// Returns the size (in bytes) of this SRAM's banks.
    pub const fn bank_size(&self) -> usize {
        BYTES_PER_BANK
    }
}