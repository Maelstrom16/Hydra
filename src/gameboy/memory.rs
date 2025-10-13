mod cartmbc;
mod consmbc;
use crate::{common::errors::HydraIOError, gameboy::Model};
use std::{cell::RefCell, fs, ops::Index, sync::RwLock};

// Header Registers
pub const TITLE_ADDRESS: usize = 0x0134;
pub const NEW_LICENSEE_CODE_ADDRESS: usize = 0x0144;
pub const HARDWARE_ADDRESS: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const OLD_LICENSEE_CODE_ADDRESS: usize = 0x014B;
pub const HEADER_CHECKSUM_ADDRESS: usize = 0x014D;

// I/O Registers
pub const P1: usize = 0xFF00;
pub const SB: usize = 0xFF01;
pub const SC: usize = 0xFF02;
pub const DIV: usize = 0xFF04;
pub const TIMA: usize = 0xFF05;
pub const TMA: usize = 0xFF06;
pub const TAC: usize = 0xFF07;
pub const IF: usize = 0xFF0F;
pub const NR10: usize = 0xFF10;
pub const NR11: usize = 0xFF11;
pub const NR12: usize = 0xFF12;
pub const NR13: usize = 0xFF13;
pub const NR14: usize = 0xFF14;
pub const NR21: usize = 0xFF16;
pub const NR22: usize = 0xFF17;
pub const NR23: usize = 0xFF18;
pub const NR24: usize = 0xFF19;
pub const NR30: usize = 0xFF1A;
pub const NR31: usize = 0xFF1B;
pub const NR32: usize = 0xFF1C;
pub const NR33: usize = 0xFF1D;
pub const NR34: usize = 0xFF1E;
pub const NR41: usize = 0xFF20;
pub const NR42: usize = 0xFF21;
pub const NR43: usize = 0xFF22;
pub const NR44: usize = 0xFF23;
pub const NR50: usize = 0xFF24;
pub const NR51: usize = 0xFF25;
pub const NR52: usize = 0xFF26;
pub const WAV00: usize = 0xFF30;
pub const WAV01: usize = 0xFF31;
pub const WAV02: usize = 0xFF32;
pub const WAV03: usize = 0xFF33;
pub const WAV04: usize = 0xFF34;
pub const WAV05: usize = 0xFF35;
pub const WAV06: usize = 0xFF36;
pub const WAV07: usize = 0xFF37;
pub const WAV08: usize = 0xFF38;
pub const WAV09: usize = 0xFF39;
pub const WAV10: usize = 0xFF3A;
pub const WAV11: usize = 0xFF3B;
pub const WAV12: usize = 0xFF3C;
pub const WAV13: usize = 0xFF3D;
pub const WAV14: usize = 0xFF3E;
pub const WAV15: usize = 0xFF3F;
pub const LCDC: usize = 0xFF40;
pub const STAT: usize = 0xFF41;
pub const SCY: usize = 0xFF42;
pub const SCX: usize = 0xFF43;
pub const LY: usize = 0xFF44;
pub const LYC: usize = 0xFF45;
pub const DMA: usize = 0xFF46;
pub const BGP: usize = 0xFF47;
pub const OBP0: usize = 0xFF48;
pub const OBP1: usize = 0xFF49;
pub const WY: usize = 0xFF4A;
pub const WX: usize = 0xFF4B;
pub const KEY0: usize = 0xFF4C;
pub const KEY1: usize = 0xFF4D;
pub const VBK: usize = 0xFF4F;
pub const BOOT: usize = 0xFF50;
pub const HDMA1: usize = 0xFF51;
pub const HDMA2: usize = 0xFF52;
pub const HDMA3: usize = 0xFF53;
pub const HDMA4: usize = 0xFF54;
pub const HDMA5: usize = 0xFF55;
pub const RP: usize = 0xFF56;
pub const BCPS: usize = 0xFF68;
pub const BPCD: usize = 0xFF69;
pub const OCPS: usize = 0xFF6A;
pub const OCPD: usize = 0xFF6B;
pub const OPRI: usize = 0xFF6C;
pub const SVBK: usize = 0xFF70;
pub const PCM12: usize = 0xFF76;
pub const PCM34: usize = 0xFF77;
pub const IE: usize = 0xFFFF;

pub struct Memory {
    cartridge: Option<Box<dyn cartmbc::CartMemoryBankController>>,
    console: Box<dyn consmbc::ConsMemoryBankController>,
    data_bus: RwLock<u8>,
}

impl Memory {
    pub fn from_rom_and_model(rom: Box<[u8]>, model: Model) -> Result<Memory, HydraIOError> {
        let result_cart = Memory {
            cartridge: Some(cartmbc::from_rom(rom)?),
            console: consmbc::from_model(model)?,
            data_bus: RwLock::new(0),
        };
        Ok(result_cart)
    }

    pub fn hot_swap_rom(&mut self, rom: Box<[u8]>) -> Result<(), HydraIOError> {
        self.cartridge = Some(cartmbc::from_rom(rom)?);
        Ok(())
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        let read_result = match address {
            0x0000..0x8000 => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_rom_u8(address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..0xA000 => self.console.read_vram_u8((address - 0x8000) as usize),
            0xA000..0xC000 => {
                if let Some(valid_cart) = &self.cartridge {
                    valid_cart.read_ram_u8((address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..0xE000 => self.console.read_wram_u8((address - 0xC000) as usize),
            0xE000..0xFE00 => self.console.read_wram_u8((address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00.. => panic!("OAM / IO / HRAM not yet implemented"),
        };
        match read_result {
            Ok(value) => *self.data_bus.write().unwrap() = value,
            Err(e) => match e {
                HydraIOError::OpenBusAccess => {}
                _ => panic!("Error reading from memory.\n{}", e),
            },
        }

        return *self.data_bus.read().unwrap();
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> () {
        *self.data_bus.write().unwrap() = value;
        let write_result = match address {
            0x0000..0x8000 => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_rom_u8(value, address as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0x8000..0xA000 => self.console.write_vram_u8(value, (address - 0x8000) as usize),
            0xA000..0xC000 => {
                if let Some(valid_cart) = &mut self.cartridge {
                    valid_cart.write_ram_u8(value, (address - 0xA000) as usize)
                } else {
                    Err(HydraIOError::OpenBusAccess)
                }
            }
            0xC000..0xE000 => self.console.write_wram_u8(value, (address - 0xC000) as usize),
            0xE000..0xFE00 => self.console.write_wram_u8(value, (address - 0xE000) as usize), // Echo RAM mirrors WRAM
            0xFE00.. => panic!("OAM / IO / HRAM not yet implemented"),
        };
        if let Err(e) = write_result {
            panic!("Error writing to memory.\n{}", e);
        }
    }
}
