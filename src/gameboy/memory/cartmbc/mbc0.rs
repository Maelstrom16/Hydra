use crate::common::errors::HydraIOError;
use crate::gameboy::memory::cartmbc;

pub struct MBC0 {
    rom: Box<[u8]>,
    ram: Vec<u8>,
}

impl MBC0 {
    pub fn from_rom(rom: Box<[u8]>) -> Result<Self, HydraIOError> {
        Ok(MBC0 {
            ram: Vec::with_capacity(cartmbc::get_ram_size(&rom)? as usize),
            rom,
        })
    }
}

impl cartmbc::CartMemoryBankController for MBC0 {
    fn read_rom_u8(&self, address: usize) -> Result<u8, HydraIOError> {
        Ok(self.rom[address])
    }
    fn read_ram_u8(&self, address: usize) -> Result<u8, HydraIOError> {
        match cartmbc::get_ram_size(&self.rom)? {
            0x000 => Err(HydraIOError::OpenBusAccess),
            0x800 => Ok(self.ram[(address - 0xA000) % 0x800]),
            _ => Ok(self.ram[address - 0xA000]),
        }
    }
    fn write_rom_u8(&mut self, _value: u8, _address: usize) -> Result<(), HydraIOError> {
        Ok(())
    }
    fn write_ram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError> {
        match cartmbc::get_ram_size(&self.rom)? {
            0x000 => Ok(()),
            0x800 => {
                self.ram[(address - 0xA000) % 0x800] = value;
                Ok(())
            }
            _ => {
                self.ram[address - 0xA000] = value;
                Ok(())
            }
        }
    }
}
