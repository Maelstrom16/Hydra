use crate::{common::errors::HydraIOError, gameboy::memory::rom::Rom};

pub struct Sram(Box<[[u8; Sram::BYTES_PER_BANK]]>);

impl Sram {
    /// The number of bytes in one bank of cartridge ROM.
    pub const BYTES_PER_BANK: usize = 0x2000;
    pub const ADDRESS_OFFSET: usize = 0xA000;

    /// Constructs a new SRAM using the size specified by a cartridge ROM.
    pub fn from_rom(rom: &Rom) -> Result<Self, HydraIOError> {
        Ok(Sram(vec![[0x00; Sram::BYTES_PER_BANK]; rom.get_ram_bank_count()?].into_boxed_slice()))
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
}