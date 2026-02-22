use crate::common::bit::BitVec;
use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::mbc;
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::Rom;

pub struct MBC2 {
    rom: Rom,
    ram: [u8; 0x200],

    ram_enabled: bool,
    rom_bank: u8,
}

impl MBC2 {
    pub fn from_rom(rom: Rom) -> Result<Self, HydraIOError> {
        Ok(MBC2 {
            ram: [0x00; 0x200],
            rom,

            ram_enabled: false,
            rom_bank: 1,
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - Rom::BYTES_PER_BANK as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> usize {
        address as usize & 0x1FF
    }
}

impl mbc::MemoryBankController for MBC2 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.ram_enabled {
            Ok(self.ram[self.localize_ram_address(address)] & 0xF)
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address.test_bit(8) {
            true => {self.rom_bank = u8::max(1, value & 0b1111)}
            false => {self.ram_enabled = value & 0xF == 0xA}
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.ram_enabled {
            Ok(self.ram[self.localize_ram_address(address)] = value & 0xF)
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}
