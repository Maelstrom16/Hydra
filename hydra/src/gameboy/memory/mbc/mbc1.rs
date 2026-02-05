use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::mbc;
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::Rom;

pub struct MBC1 {
    rom: Rom,
    ram: Sram,

    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    dynamic_banking: bool,
}

impl MBC1 {
    pub fn from_rom(rom: Rom) -> Result<Self, HydraIOError> {
        Ok(MBC1 {
            ram: Sram::from_rom(&rom)?,
            rom,

            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            dynamic_banking: false
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        let bank_upper = match self.dynamic_banking {
            true => self.ram_bank << 5,
            false => 0
        };
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: (0 | bank_upper) as usize},
            0x4000..=0x7FFF => BankedAddress {address: address - Rom::BYTES_PER_BANK as u16, bank: (self.rom_bank | bank_upper) as usize % self.rom.get_bank_count()},
            _ => panic!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        let bank = match self.dynamic_banking {
            true => self.ram_bank as usize % self.ram.get_bank_count(),
            false => 0
        };
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - Sram::ADDRESS_OFFSET as u16, bank},
            _ => panic!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for MBC1 {
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
            0x0000..=0x1FFF => {self.ram_enabled = value & 0xF == 0xA}
            0x2000..=0x3FFF => {self.rom_bank = u8::max(1, value & 0b11111)}
            0x4000..=0x5FFF => {self.ram_bank = value & 0b11}
            0x6000..=0x7FFF => {self.dynamic_banking = value & 0b1 == 0b1}
            _ => panic!("Invalid ROM address")
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
