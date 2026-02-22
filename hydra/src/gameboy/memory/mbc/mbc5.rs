use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::mbc;
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::Rom;

pub struct MBC5 {
    rom: Rom,
    ram: Sram,

    ram_enabled: bool,
    rom_bank: u16,
    ram_bank: u8,

    rumble: bool,
}

impl MBC5 {
    pub fn from_rom(rom: Rom) -> Result<Self, HydraIOError> {
        Ok(MBC5 {
            ram: Sram::from_rom(&rom)?,
            rom,

            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            
            // TODO: Implement rumble for controllers
            rumble: false
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - Rom::BYTES_PER_BANK as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - Sram::ADDRESS_OFFSET as u16, bank: self.ram_bank as usize % self.ram.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for MBC5 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.ram_enabled {
            let BankedAddress { address, bank } = self.localize_ram_address(address);
            Ok(self.ram.read_bank(address, bank))
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address {
            0x0000..=0x1FFF => {self.ram_enabled = value == 0x0A}
            0x2000..=0x2FFF => {self.rom_bank = (self.rom_bank & 0b100000000) | value as u16}
            0x3000..=0x3FFF => {self.rom_bank = (self.rom_bank & 0b011111111) | ((value as u16 & 1) << 8)}
            0x4000..=0x5FFF => {
                self.ram_bank = if self.rumble {
                    value & 0b0111
                } else {
                    value & 0b1111
                }
            }
            0x6000..=0x7FFF => { /* Do nothing */ }
            _ => unimplemented!("Attempted to write {:#04X} to invalid ROM address {:#06X}", value, address)
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.ram_enabled {
            let BankedAddress { address, bank } = self.localize_ram_address(address);
            Ok(self.ram.write_bank(value, address, bank))
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}
