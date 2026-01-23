pub mod deserialized;

use std::{array, rc::Rc};

use hydra_macros::TrijectiveMMIO;

use crate::{common::bit::{MaskedBitSet, WriteBehavior}, gameboy::{GBRevision, Model}};

pub const ADDRESS_OFFSET: u16 = 0xFF00;

#[derive(TrijectiveMMIO)]
#[repr(u16)]
pub enum MMIO {
    P1 = 0xFF00,
    SB = 0xFF01,
    SC = 0xFF02,
    DIV = 0xFF04,
    TIMA = 0xFF05,
    TMA = 0xFF06,
    TAC = 0xFF07,
    IF = 0xFF0F,
    NR10 = 0xFF10,
    NR11 = 0xFF11,
    NR12 = 0xFF12,
    NR13 = 0xFF13,
    NR14 = 0xFF14,
    NR21 = 0xFF16,
    NR22 = 0xFF17,
    NR23 = 0xFF18,
    NR24 = 0xFF19,
    NR30 = 0xFF1A,
    NR31 = 0xFF1B,
    NR32 = 0xFF1C,
    NR33 = 0xFF1D,
    NR34 = 0xFF1E,
    NR41 = 0xFF20,
    NR42 = 0xFF21,
    NR43 = 0xFF22,
    NR44 = 0xFF23,
    NR50 = 0xFF24,
    NR51 = 0xFF25,
    NR52 = 0xFF26,
    WAV00 = 0xFF30,
    WAV01 = 0xFF31,
    WAV02 = 0xFF32,
    WAV03 = 0xFF33,
    WAV04 = 0xFF34,
    WAV05 = 0xFF35,
    WAV06 = 0xFF36,
    WAV07 = 0xFF37,
    WAV08 = 0xFF38,
    WAV09 = 0xFF39,
    WAV10 = 0xFF3A,
    WAV11 = 0xFF3B,
    WAV12 = 0xFF3C,
    WAV13 = 0xFF3D,
    WAV14 = 0xFF3E,
    WAV15 = 0xFF3F,
    LCDC = 0xFF40,
    STAT = 0xFF41,
    SCY = 0xFF42,
    SCX = 0xFF43,
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A,
    WX = 0xFF4B,
    KEY0 = 0xFF4C,
    KEY1 = 0xFF4D,
    VBK = 0xFF4F,
    BOOT = 0xFF50,
    HDMA1 = 0xFF51,
    HDMA2 = 0xFF52,
    HDMA3 = 0xFF53,
    HDMA4 = 0xFF54,
    HDMA5 = 0xFF55,
    RP = 0xFF56,
    BCPS = 0xFF68,
    BCPD = 0xFF69,
    OCPS = 0xFF6A,
    OCPD = 0xFF6B,
    OPRI = 0xFF6C,
    SVBK = 0xFF70,
    PCM12 = 0xFF76,
    PCM34 = 0xFF77,
    IE = 0xFFFF
}

pub type GBReg = MaskedBitSet<u8>;

pub struct IOMap {
    registers: [Rc<GBReg>; MMIO::VARIANT_COUNT]
}

impl IOMap {
    pub fn new(model: Model) -> Self {
        IOMap {
            registers: array::from_fn(|index| match MMIO::from_local(index) {
                // Define default values for all registers and models
                MMIO::P1 => GBReg::new(0xCF, 0b00111111, 0b00110000, WriteBehavior::Standard),
                MMIO::SB => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::SC => match model {
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0x7E, 0b10000001, 0b10000001, WriteBehavior::Standard),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x7F, 0b10000011, 0b10000011, WriteBehavior::Standard),
                },
                MMIO::DIV => GBReg::new(match model { 
                    Model::GameBoy(Some(GBRevision::DMG0)) => 0x18,
                    Model::GameBoy(_) => 0xAB,
                    Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
                }, 0b11111111, 0b11111111, WriteBehavior::ResetOnWrite),
                MMIO::TIMA => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::TMA => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::TAC => GBReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
                MMIO::IF => GBReg::new(0xE1, 0b00011111, 0b00011111, WriteBehavior::Standard),
                MMIO::NR10 => GBReg::new(0x80, 0b01111111, 0b01111111, WriteBehavior::Standard),
                MMIO::NR11 => GBReg::new(0xBF, 0b11000000, 0b11111111, WriteBehavior::Standard),
                MMIO::NR12 => GBReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR13 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
                MMIO::NR14 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
                MMIO::NR21 => GBReg::new(0x3F, 0b11000000, 0b11111111, WriteBehavior::Standard),
                MMIO::NR22 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR23 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
                MMIO::NR24 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
                MMIO::NR30 => GBReg::new(0x7F, 0b10000000, 0b10000000, WriteBehavior::Standard),
                MMIO::NR31 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
                MMIO::NR32 => GBReg::new(0x9F, 0b01100000, 0b01100000, WriteBehavior::Standard),
                MMIO::NR33 => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR34 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
                MMIO::NR41 => GBReg::new(0xFF, 0b00000000, 0b00111111, WriteBehavior::Standard),
                MMIO::NR42 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR43 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR44 => GBReg::new(0xBF, 0b01000000, 0b11000000, WriteBehavior::Standard),
                MMIO::NR50 => GBReg::new(0x77, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR51 => GBReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::NR52 => GBReg::new(match model {
                    Model::GameBoy(_) => 0xF1,
                    Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0xF0,
                }, 0b10001111, 0b00001111, WriteBehavior::Standard),
                MMIO::WAV00 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV01 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV02 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV03 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV04 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV05 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV06 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV07 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV08 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV09 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV10 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV11 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV12 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV13 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV14 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WAV15 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::LCDC => GBReg::new(0x91, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::STAT => GBReg::new(match model {
                    Model::GameBoy(Some(GBRevision::DMG0)) => 0x81,
                    Model::GameBoy(_) => 0x85,
                    Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
                }, 0b01111111, 0b01111000, WriteBehavior::Standard),
                MMIO::SCY => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::SCX => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::LY => GBReg::new(match model {
                    Model::GameBoy(Some(GBRevision::DMG0)) => 0x91,
                    Model::GameBoy(_) => 0x00,
                    Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
                }, 0b11111111, 0b00000000, WriteBehavior::Standard),
                MMIO::LYC => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::DMA => GBReg::new(match model {
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => 0xFF,
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0x00,
                }, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::BGP => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFC, 0b11111111, 0b11111111, WriteBehavior::Standard),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
                },
                MMIO::OBP0 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
                },
                MMIO::OBP1 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
                },
                MMIO::WY => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::WX => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
                MMIO::KEY0 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(rand::random(), 0b00000100, 0b00000000, WriteBehavior::Standard), // TODO: Value is supposed to be based on header contents, Allow writing during boot ROM if included
                }, 
                MMIO::KEY1 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x7E, 0b10000001, 0b00000001, WriteBehavior::Standard),
                }, 
                MMIO::VBK => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFE, 0b00000001, 0b00000001, WriteBehavior::Standard),
                }, 
                MMIO::BOOT => GBReg::new(0xFF, 0b00000000, 0b00000001, WriteBehavior::UnmapBootRom), // TODO: Verify write behavior
                MMIO::HDMA1 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
                }, 
                MMIO::HDMA2 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
                }, 
                MMIO::HDMA3 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure upper three bits are ignored
                }, 
                MMIO::HDMA4 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
                }, 
                MMIO::HDMA5 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Ensure proper read behavior
                }, 
                MMIO::RP => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x3E, 0b11000011, 0b00000010, WriteBehavior::Standard),
                },
                MMIO::BCPS => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
                },
                MMIO::BCPD => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
                },
                MMIO::OCPS => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
                },
                MMIO::OCPD => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
                },
                MMIO::OPRI => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000001, 0b00000001, WriteBehavior::Standard), // TODO: Verify startup value and masks
                },
                MMIO::SVBK => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
                },
                MMIO::PCM12 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
                },
                MMIO::PCM34 => match model { 
                    Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                    Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
                },
                MMIO::IE => GBReg::new(0x00, 0b00011111, 0b00011111, WriteBehavior::Standard),
            })
        }
    }
}

impl IOMap {
    pub fn read(&self, address: u16) -> u8 {
        self.registers[address as usize].read()
    }

    pub fn write(&mut self, value: u8, address: u16) {
        self.registers[address as usize].write(value)
    }

    pub fn clone_pointer(&self, mmio: MMIO) -> Rc<GBReg> {
        self.registers[mmio.to_local()].clone()
    }

    const fn localize_address(address: u16) -> usize {
        MMIO::global_to_local(address)
    }
}