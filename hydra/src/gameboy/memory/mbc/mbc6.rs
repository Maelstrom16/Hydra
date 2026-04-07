use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};

pub struct MBC6 {
    rom: Rom<0x2000>,
    ram: Sram<0x1000>,

    rom_banks: [u16; 2],
    rom_bank_select: [RomBankMapping; 2],
    flash_read_enabled: bool,
    flash_write_enabled: bool,
    ram_banks: [u8; 2],
    ram_enabled: bool,
}

impl MBC6 {
    pub fn from_header(header: RomHeader) -> Result<Self, HydraIOError> {
        Ok(MBC6 {
            ram: Sram::from_header(&header)?,
            rom: header.into_rom(),

            rom_banks: [2, 3],
            rom_bank_select: [RomBankMapping::Rom; 2],
            flash_read_enabled: false,
            flash_write_enabled: false,
            ram_banks: [0, 1],
            ram_enabled: false,
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x1FFF => BankedAddress {address: address, bank: 0},
            0x2000..=0x3FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: 1},
            0x4000..=0x5FFF => BankedAddress {address: address - (self.rom.bank_size() * 2) as u16, bank: self.rom_banks[0] as usize % self.rom.get_bank_count()},
            0x6000..=0x7FFF => BankedAddress {address: address - (self.rom.bank_size() * 3) as u16, bank: self.rom_banks[1] as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        let bank = self.ram_banks[match address {
            0xA000..=0xAFFF => 0,
            0xB000..=0xBFFF => 1,
            _ => unimplemented!("Attempted to localize invalid RAM address {}", address)
        }] as usize % self.ram.get_bank_count();

        BankedAddress {address: address - sram::ADDRESS_OFFSET as u16, bank}
    }
}

impl mbc::MemoryBankController for MBC6 {
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
            0x0000..=0x03FF => match value {
                0x00 => self.ram_enabled = false,
                0x0A => self.ram_enabled = true,
                _ => { /* Leave RAM in current state */ }
            }
            0x0400..=0x07FF => {self.ram_banks[0] = value & 0b1111}
            0x0800..=0x0BFF => {self.ram_banks[1] = value & 0b1111}
            0x0C00..=0x0FFF => {self.flash_read_enabled = value & 1 == 1}
            0x1000..=0x1FFF => {self.flash_write_enabled = value & 1 == 1}
            0x2000..=0x27FF => {self.rom_banks[0] = value as u16}
            0x2800..=0x2FFF => match value {
                0x00 => self.rom_bank_select[0] = RomBankMapping::Rom,
                0x08 => self.rom_bank_select[0] = RomBankMapping::Flash,
                _ => { /* Leave ROM bank A in current state */ }
            }
            0x3000..=0x37FF => {self.rom_banks[1] = value as u16}
            0x3800..=0x3FFF => match value {
                0x00 => self.rom_bank_select[1] = RomBankMapping::Rom,
                0x08 => self.rom_bank_select[1] = RomBankMapping::Flash,
                _ => { /* Leave ROM bank B in current state */ }
            }
            0x4000..=0x7FFF => { /* Do nothing */ }
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

#[derive(Copy, Clone)]
enum RomBankMapping {
    Rom = 0,
    Flash = 1
}