use std::{fs, path::{Path, PathBuf}, rc::Rc};

use crate::{common::errors::HydraIOError, emulator::{SavePath, gameboy::memory::rom::RomHeader}};

pub const ADDRESS_OFFSET: usize = 0xA000;

pub struct Sram<const BYTES_PER_BANK: usize> {
    data: Box<[[u8; BYTES_PER_BANK]]>,
    save_path: PathBuf,
}

impl<const BYTES_PER_BANK: usize> Sram<BYTES_PER_BANK> {
    /// Constructs a new SRAM using the size specified by a cartridge ROM.
    pub fn from_header(header: &RomHeader) -> Result<Self, HydraIOError> {
        let sram_slice = fs::read(header.save_path()).map_or(vec![[0x00; BYTES_PER_BANK]; header.get_ram_bank_count()?], |save_file| save_file.as_chunks::<BYTES_PER_BANK>().0.to_vec()).into_boxed_slice();
        Ok(Sram {
            data: sram_slice,
            save_path: header.save_path().to_owned(),
        })
    }

    /// Reads the byte from this SRAM at the provided address and bank. 
    pub fn read_bank(&self, address: u16, bank: usize) -> u8 {
        self.data[bank][address as usize]
    }

    /// Writes a value to the byte at the provided address and bank in this SRAM. 
    pub fn write_bank(&mut self, value: u8, address: u16, bank: usize) {
        self.data[bank][address as usize] = value;
    }

    /// Returns the number of banks this SRAM consists of.
    pub fn get_bank_count(&self) -> usize {
        self.data.len()
    }

    pub fn save_to_file(&self) -> Result<(), HydraIOError> {
        Ok(fs::write(&self.save_path, self.data.as_flattened())?)
    }

    /// Returns the size (in bytes) of this SRAM's banks.
    pub const fn bank_size(&self) -> usize {
        BYTES_PER_BANK
    }
}