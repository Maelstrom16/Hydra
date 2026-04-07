use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};

pub struct MBC0 {
    rom: Rom<0x4000>,
    ram: Sram<0x2000>,
}

impl MBC0 {
    pub fn from_header(header: RomHeader) -> Result<Self, HydraIOError> {
        Ok(MBC0 {
            ram: Sram::from_header(&header)?,
            rom: header.into_rom(),
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: 1},
            _ => panic!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - sram::ADDRESS_OFFSET as u16, bank: 0},
            _ => panic!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for MBC0 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_ram_address(address);
        Ok(self.ram.read_bank(address, bank))
    }
    fn write_rom_u8(&mut self, _value: u8, _address: u16) -> Result<(), HydraIOError> {
        Ok(())
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        let BankedAddress { address, bank } = self.localize_ram_address(address);
        Ok(self.ram.write_bank(value, address, bank))
    }
}
