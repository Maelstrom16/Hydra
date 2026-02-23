use std::time::{Duration, Instant};

use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::mbc;
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::Rom;
use crate::{deserialize, serialize};

pub struct MBC3 {
    rom: Rom,
    ram: Sram,

    ram_area_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,

    rtc: Option<RealTimeClock>,
    rtc_latch: u8,
}

impl MBC3 {
    pub fn from_rom(rom: Rom) -> Result<Self, HydraIOError> {
        Ok(MBC3 {
            ram: Sram::from_rom(&rom)?,
            rtc: rom.get_rtc(),
            rom,

            ram_area_enabled: false,
            rom_bank: 1,
            ram_bank: 0,

            rtc_latch: 0xFF,
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - Rom::BYTES_PER_BANK as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => panic!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - Sram::ADDRESS_OFFSET as u16, bank: self.ram_bank as usize % self.ram.get_bank_count()},
            _ => panic!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for MBC3 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.ram_area_enabled {
            match (self.ram_bank, &self.rtc) {
                (0x00..=0x07, _) => {
                    let BankedAddress { address, bank } = self.localize_ram_address(address);
                    Ok(self.ram.read_bank(address, bank))
                }
                (0x08, Some(rtc)) => Ok(rtc.latched_seconds),
                (0x09, Some(rtc)) => Ok(rtc.latched_minutes),
                (0x0A, Some(rtc)) => Ok(rtc.latched_hours),
                (0x0B, Some(rtc)) => Ok((rtc.latched_days & 0xFF) as u8),
                (0x0C, Some(rtc)) => Ok(serialize!(
                    (rtc.day_carry as u8) =>> 7;
                    (rtc.halted as u8) =>> 6;
                    0b00111110;
                    (((rtc.latched_days & 0b100000000) >> 8) as u8) =>> 0;
                )),
                _ => Err(HydraIOError::OpenBusAccess)
            }
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address {
            0x0000..=0x1FFF => match value {
                0x00 => self.ram_area_enabled = false,
                0x0A => self.ram_area_enabled = true,
                _ => { /* Leave RAM in current state */ }
            }
            0x2000..=0x3FFF => {self.rom_bank = u8::max(1, value & 0b1111111)}
            0x4000..=0x5FFF => if (0x00..=0x0C).contains(&value) {
                self.ram_bank = value;
            }
            0x6000..=0x7FFF => {
                if let Some(rtc) = &mut self.rtc && value == 0x01 && self.rtc_latch == 0x00 {
                    rtc.latch();
                }
                self.rtc_latch = value;
            }
            _ => panic!("Invalid ROM address")
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.ram_area_enabled {
            match (self.ram_bank, &mut self.rtc) {
                (0x00..=0x07, _) => {
                    let BankedAddress { address, bank } = self.localize_ram_address(address);
                    Ok(self.ram.write_bank(value, address, bank))
                }
                (0x08, Some(rtc)) => Ok(rtc.latched_seconds = value),
                (0x09, Some(rtc)) => Ok(rtc.latched_minutes = value),
                (0x0A, Some(rtc)) => Ok(rtc.latched_hours = value),
                (0x0B, Some(rtc)) => Ok(rtc.latched_days = (rtc.latched_days & 0x100) | value as u16),
                (0x0C, Some(rtc)) => {
                    deserialize!(value;
                        7 as bool =>> (rtc.day_carry);
                        6 as bool =>> (rtc.halted);
                        0 =>> days_hi;
                    );
                    rtc.latched_days = (rtc.latched_days & 0xFF) | ((days_hi as u16) << 8);
                    Ok(())
                },
                _ => Err(HydraIOError::OpenBusAccess)
            }
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}

pub struct RealTimeClock {
    pub(self) day_carry: bool,
    pub(self) halted: bool,

    timestamp: Instant,
    pub(self) latched_seconds: u8,
    pub(self) latched_minutes: u8,
    pub(self) latched_hours: u8,
    pub(self) latched_days: u16,
}

impl RealTimeClock {
    pub fn new() -> Self {
        RealTimeClock { 
            day_carry: false,
            halted: true,

            timestamp: Instant::now(),
            latched_seconds: 0,
            latched_minutes: 0,
            latched_hours: 0,
            latched_days: 0,
        }
    }

    pub fn latch(&mut self) {
        let latched = if self.halted {
            Duration::ZERO
        } else {
            self.timestamp.elapsed()
        };

        self.latched_seconds = latched.as_secs() as u8 % 60;
        self.latched_minutes = (latched.as_secs() / 60) as u8 % 60;
        self.latched_hours = (latched.as_secs() / 3600) as u8 % 24;
        self.latched_days = (latched.as_secs() / 86400) as u16 % 512;
    }
}