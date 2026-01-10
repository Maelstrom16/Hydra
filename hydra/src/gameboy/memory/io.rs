mod deserialized;

use std::{
    array, ops::{Index, IndexMut}, rc::Rc
};

use crate::{common::bit::{MaskedBitSet, WriteBehavior}, gameboy::{GBRevision, Model}};

pub const ADDRESS_OFFSET: usize = 0xFF00;
// I/O Register Addresses
pub const P1: usize = 0xFF00 - ADDRESS_OFFSET;
pub const SB: usize = 0xFF01 - ADDRESS_OFFSET;
pub const SC: usize = 0xFF02 - ADDRESS_OFFSET;
pub const DIV: usize = 0xFF04 - ADDRESS_OFFSET;
pub const TIMA: usize = 0xFF05 - ADDRESS_OFFSET;
pub const TMA: usize = 0xFF06 - ADDRESS_OFFSET;
pub const TAC: usize = 0xFF07 - ADDRESS_OFFSET;
pub const IF: usize = 0xFF0F - ADDRESS_OFFSET;
pub const NR10: usize = 0xFF10 - ADDRESS_OFFSET;
pub const NR11: usize = 0xFF11 - ADDRESS_OFFSET;
pub const NR12: usize = 0xFF12 - ADDRESS_OFFSET;
pub const NR13: usize = 0xFF13 - ADDRESS_OFFSET;
pub const NR14: usize = 0xFF14 - ADDRESS_OFFSET;
pub const NR21: usize = 0xFF16 - ADDRESS_OFFSET;
pub const NR22: usize = 0xFF17 - ADDRESS_OFFSET;
pub const NR23: usize = 0xFF18 - ADDRESS_OFFSET;
pub const NR24: usize = 0xFF19 - ADDRESS_OFFSET;
pub const NR30: usize = 0xFF1A - ADDRESS_OFFSET;
pub const NR31: usize = 0xFF1B - ADDRESS_OFFSET;
pub const NR32: usize = 0xFF1C - ADDRESS_OFFSET;
pub const NR33: usize = 0xFF1D - ADDRESS_OFFSET;
pub const NR34: usize = 0xFF1E - ADDRESS_OFFSET;
pub const NR41: usize = 0xFF20 - ADDRESS_OFFSET;
pub const NR42: usize = 0xFF21 - ADDRESS_OFFSET;
pub const NR43: usize = 0xFF22 - ADDRESS_OFFSET;
pub const NR44: usize = 0xFF23 - ADDRESS_OFFSET;
pub const NR50: usize = 0xFF24 - ADDRESS_OFFSET;
pub const NR51: usize = 0xFF25 - ADDRESS_OFFSET;
pub const NR52: usize = 0xFF26 - ADDRESS_OFFSET;
pub const WAV00: usize = 0xFF30 - ADDRESS_OFFSET;
pub const WAV01: usize = 0xFF31 - ADDRESS_OFFSET;
pub const WAV02: usize = 0xFF32 - ADDRESS_OFFSET;
pub const WAV03: usize = 0xFF33 - ADDRESS_OFFSET;
pub const WAV04: usize = 0xFF34 - ADDRESS_OFFSET;
pub const WAV05: usize = 0xFF35 - ADDRESS_OFFSET;
pub const WAV06: usize = 0xFF36 - ADDRESS_OFFSET;
pub const WAV07: usize = 0xFF37 - ADDRESS_OFFSET;
pub const WAV08: usize = 0xFF38 - ADDRESS_OFFSET;
pub const WAV09: usize = 0xFF39 - ADDRESS_OFFSET;
pub const WAV10: usize = 0xFF3A - ADDRESS_OFFSET;
pub const WAV11: usize = 0xFF3B - ADDRESS_OFFSET;
pub const WAV12: usize = 0xFF3C - ADDRESS_OFFSET;
pub const WAV13: usize = 0xFF3D - ADDRESS_OFFSET;
pub const WAV14: usize = 0xFF3E - ADDRESS_OFFSET;
pub const WAV15: usize = 0xFF3F - ADDRESS_OFFSET;
pub const LCDC: usize = 0xFF40 - ADDRESS_OFFSET;
pub const STAT: usize = 0xFF41 - ADDRESS_OFFSET;
pub const SCY: usize = 0xFF42 - ADDRESS_OFFSET;
pub const SCX: usize = 0xFF43 - ADDRESS_OFFSET;
pub const LY: usize = 0xFF44 - ADDRESS_OFFSET;
pub const LYC: usize = 0xFF45 - ADDRESS_OFFSET;
pub const DMA: usize = 0xFF46 - ADDRESS_OFFSET;
pub const BGP: usize = 0xFF47 - ADDRESS_OFFSET;
pub const OBP0: usize = 0xFF48 - ADDRESS_OFFSET;
pub const OBP1: usize = 0xFF49 - ADDRESS_OFFSET;
pub const WY: usize = 0xFF4A - ADDRESS_OFFSET;
pub const WX: usize = 0xFF4B - ADDRESS_OFFSET;
pub const KEY0: usize = 0xFF4C - ADDRESS_OFFSET;
pub const KEY1: usize = 0xFF4D - ADDRESS_OFFSET;
pub const VBK: usize = 0xFF4F - ADDRESS_OFFSET;
pub const BOOT: usize = 0xFF50 - ADDRESS_OFFSET;
pub const HDMA1: usize = 0xFF51 - ADDRESS_OFFSET;
pub const HDMA2: usize = 0xFF52 - ADDRESS_OFFSET;
pub const HDMA3: usize = 0xFF53 - ADDRESS_OFFSET;
pub const HDMA4: usize = 0xFF54 - ADDRESS_OFFSET;
pub const HDMA5: usize = 0xFF55 - ADDRESS_OFFSET;
pub const RP: usize = 0xFF56 - ADDRESS_OFFSET;
pub const BCPS: usize = 0xFF68 - ADDRESS_OFFSET;
pub const BCPD: usize = 0xFF69 - ADDRESS_OFFSET;
pub const OCPS: usize = 0xFF6A - ADDRESS_OFFSET;
pub const OCPD: usize = 0xFF6B - ADDRESS_OFFSET;
pub const OPRI: usize = 0xFF6C - ADDRESS_OFFSET;
pub const SVBK: usize = 0xFF70 - ADDRESS_OFFSET;
pub const PCM12: usize = 0xFF76 - ADDRESS_OFFSET;
pub const PCM34: usize = 0xFF77 - ADDRESS_OFFSET;
pub const IE: usize = 0xFFFF - ADDRESS_OFFSET - 0x7F; // To compensate for HRAM

pub type GBReg = MaskedBitSet<u8>;

pub struct IOMap {
    registers: [Rc<GBReg>; 0x80 + 1],
}

impl IOMap {
    pub fn new(model: Model) -> Self {
        IOMap { registers: array::from_fn(|i| match i {
            // Define default values for all registers and models
            P1 => GBReg::new(0xCF, 0b00111111, 0b00110000, WriteBehavior::Standard),
            SB => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            SC => match model {
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0x7E, 0b10000001, 0b10000001, WriteBehavior::Standard),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x7F, 0b10000011, 0b10000011, WriteBehavior::Standard),
            },
            DIV => GBReg::new(match model { 
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x18,
                Model::GameBoy(_) => 0xAB,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b11111111, 0b11111111, WriteBehavior::ResetOnWrite),
            TIMA => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            TMA => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            TAC => GBReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
            IF => GBReg::new(0xE1, 0b00011111, 0b00011111, WriteBehavior::Standard),
            NR10 => GBReg::new(0x80, 0b01111111, 0b01111111, WriteBehavior::Standard),
            NR11 => GBReg::new(0xBF, 0b11000000, 0b11111111, WriteBehavior::Standard),
            NR12 => GBReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR13 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR14 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR21 => GBReg::new(0x3F, 0b11000000, 0b11111111, WriteBehavior::Standard),
            NR22 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR23 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR24 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR30 => GBReg::new(0x7F, 0b10000000, 0b10000000, WriteBehavior::Standard),
            NR31 => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR32 => GBReg::new(0x9F, 0b01100000, 0b01100000, WriteBehavior::Standard),
            NR33 => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR34 => GBReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR41 => GBReg::new(0xFF, 0b00000000, 0b00111111, WriteBehavior::Standard),
            NR42 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR43 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR44 => GBReg::new(0xBF, 0b01000000, 0b11000000, WriteBehavior::Standard),
            NR50 => GBReg::new(0x77, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR51 => GBReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR52 => GBReg::new(match model {
                Model::GameBoy(_) => 0xF1,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0xF0,
            }, 0b10001111, 0b00001111, WriteBehavior::Standard),
            WAV00 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV01 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV02 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV03 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV04 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV05 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV06 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV07 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV08 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV09 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV10 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV11 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV12 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV13 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV14 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV15 => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            LCDC => GBReg::new(0x91, 0b11111111, 0b11111111, WriteBehavior::Standard),
            STAT => GBReg::new(match model {
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x81,
                Model::GameBoy(_) => 0x85,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b01111111, 0b01111000, WriteBehavior::Standard),
            SCY => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            SCX => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            LY => GBReg::new(match model {
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x91,
                Model::GameBoy(_) => 0x00,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b11111111, 0b00000000, WriteBehavior::Standard),
            LYC => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            DMA => GBReg::new(match model {
                Model::GameBoy(_) | Model::SuperGameBoy(_) => 0xFF,
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0x00,
            }, 0b11111111, 0b11111111, WriteBehavior::Standard),
            BGP => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFC, 0b11111111, 0b11111111, WriteBehavior::Standard),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
            },
            OBP0 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
            },
            OBP1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new_unimplemented(),
            },
            WY => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WX => GBReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            KEY0 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(rand::random(), 0b00000100, 0b00000000, WriteBehavior::Standard), // TODO: Value is supposed to be based on header contents, Allow writing during boot ROM if included
            }, 
            KEY1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x7E, 0b10000001, 0b00000001, WriteBehavior::Standard),
            }, 
            VBK => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFE, 0b00000001, 0b00000001, WriteBehavior::Standard),
            }, 
            BOOT => GBReg::new(0xFF, 0b00000000, 0b00000001, WriteBehavior::UnmapBootRom), // TODO: Verify write behavior
            HDMA1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            }, 
            HDMA2 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
            }, 
            HDMA3 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure upper three bits are ignored
            }, 
            HDMA4 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
            }, 
            HDMA5 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Ensure proper read behavior
            }, 
            RP => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0x3E, 0b11000011, 0b00000010, WriteBehavior::Standard),
            },
            BCPS => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
            },
            BCPD => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
            },
            OCPS => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
            },
            OCPD => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
            },
            OPRI => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b00000001, 0b00000001, WriteBehavior::Standard), // TODO: Verify startup value and masks
            },
            SVBK => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
            },
            PCM12 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
            },
            PCM34 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
            },
            IE => GBReg::new(0x00, 0b00011111, 0b00011111, WriteBehavior::Standard),
            _ => GBReg::new_unimplemented()
        })}
    }
}

impl Index<usize> for IOMap {
    type Output = Rc<GBReg>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}
impl IndexMut<usize> for IOMap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}