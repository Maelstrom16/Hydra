use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};
use crate::{deserialize, serialize};

pub struct HuC3 {
    rom: Rom<0x4000>,
    ram: Sram<0x2000>,

    ram_area_mode: RamAreaMode,
    rom_bank: u8,
    ram_bank: u8,

    rtc_command: u8,
    rtc_argument: u8,
    rtc_output: u8,

    rtc_address: u8,
    rtc_memory: [u8; 0x100],
}

impl HuC3 {
    pub fn from_header(header: RomHeader) -> Result<Self, HydraIOError> {
        Ok(HuC3 {
            ram: Sram::from_header(&header)?,
            rom: header.into_rom(),

            ram_area_mode: RamAreaMode::Invalid,
            rom_bank: 1,
            ram_bank: 0,

            rtc_command: 0x0,
            rtc_argument: 0x0,
            rtc_output: 0x0,

            rtc_address: 0x00,
            rtc_memory: [0x00; 0x100]
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - sram::ADDRESS_OFFSET as u16, bank: self.ram_bank as usize % self.ram.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for HuC3 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        match self.ram_area_mode {
            RamAreaMode::SramReadOnly | RamAreaMode::SramReadWrite => {
                let BankedAddress { address, bank } = self.localize_ram_address(address);
                Ok(self.ram.read_bank(address, bank))
            }
            RamAreaMode::RtcCommandRead => {
                Ok(serialize!(
                    0b10000000;
                    (self.rtc_command) =>> [6..=4];
                    (self.rtc_output) =>> [3..=0];
                ))
            }
            RamAreaMode::RtcSemaphore => Ok(0xFF), // Always ready for next command, since internal delay is unknown
            RamAreaMode::Infrared => {
                // TODO: Implement IR transmission
                Ok(0xC0)
            }

            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address {
            0x0000..=0x1FFF => { 
            println!("MODE SWITCH");
            self.ram_area_mode = match value & 0xF {
                0x0 => RamAreaMode::SramReadOnly,
                0xA => RamAreaMode::SramReadWrite,
                0xB => RamAreaMode::RtcCommandWrite,
                0xC => RamAreaMode::RtcCommandRead,
                0xD => RamAreaMode::RtcSemaphore,
                0xE => RamAreaMode::Infrared,
                _ => RamAreaMode::Invalid,
            }}
            0x2000..=0x3FFF => {self.rom_bank = value & 0b1111111}
            0x4000..=0x5FFF => {self.ram_bank = value & 0b11}
            0x6000..=0x7FFF => {}
            _ => panic!("Invalid ROM address")
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        match self.ram_area_mode {
            RamAreaMode::SramReadWrite => {
                let BankedAddress { address, bank } = self.localize_ram_address(address);
                self.ram.write_bank(value, address, bank);
                Ok(())
            }
            RamAreaMode::RtcCommandWrite => {
                deserialize!(value;
                    [6..=4] =>> (self.rtc_command);
                    [3..=0] =>> (self.rtc_argument);
                );
                Ok(())
            }
            RamAreaMode::RtcSemaphore => {
                match self.rtc_command {
                    0x0 | 0x1 => {
                        self.rtc_output = self.rtc_memory[self.rtc_address as usize];
                        self.rtc_address = self.rtc_address.wrapping_add(self.rtc_command & 0b1);
                        Ok(())
                    }
                    0x2 | 0x3 => {
                        self.rtc_memory[self.rtc_address as usize] = self.rtc_argument;
                        self.rtc_address = self.rtc_address.wrapping_add(self.rtc_command & 0b1);
                        Ok(())
                    }
                    0x4 => {
                        self.rtc_address = (self.rtc_address & 0b11110000) | (self.rtc_argument);
                        Ok(())
                    }
                    0x5 => {
                        self.rtc_address = (self.rtc_address & 0b00001111) | (self.rtc_argument << 4);
                        Ok(())
                    }
                    0x6 => match self.rtc_argument {
                        0x0 => {
                            self.rtc_memory.copy_within(0x10..=0x15, 0x00);
                            Ok(())
                        }
                        0x1 => {
                            self.rtc_memory.copy_within(0x00..=0x05, 0x10);
                            // TODO: update alarm accordingly
                            Ok(())
                        }
                        0x2 => {
                            // Exact purpose unknown. TODO: verify behavior
                            self.rtc_output = 0x1;
                            Ok(())
                        }
                        0xE => {
                            // TODO: Implement tone generation
                            Ok(())
                        }
                        _ => Err(HydraIOError::OpenBusAccess)
                    }
                    _ => Err(HydraIOError::OpenBusAccess)
                }
            }
            RamAreaMode::Infrared => {
                // TODO: Implement IR transmission
                Ok(())
            }
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

enum RamAreaMode {
    SramReadOnly,
    SramReadWrite,
    RtcCommandWrite,
    RtcCommandRead,
    RtcSemaphore,
    Infrared,
    Invalid,
}