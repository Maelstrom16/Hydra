use std::{
    array, cell::Cell, mem::MaybeUninit, ops::{Index, IndexMut}, rc::Rc
};

use crate::gameboy::{GBRevision, Model};

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

pub struct IOMap {
    registers: [Rc<IOReg>; 0x80 + 1],
}

impl IOMap {
    pub fn new(model: Model) -> Self {
        IOMap { registers: array::from_fn(|i| match i {
            // Define default values for all registers and models
            P1 => IOReg::new(0xCF, 0b00111111, 0b00110000, WriteBehavior::Standard),
            SB => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            SC => match model {
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new(0x7E, 0b10000001, 0b10000001, WriteBehavior::Standard),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0x7F, 0b10000011, 0b10000011, WriteBehavior::Standard),
            },
            DIV => IOReg::new(match model { 
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x18,
                Model::GameBoy(_) => 0xAB,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b11111111, 0b11111111, WriteBehavior::ResetOnWrite),
            TIMA => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            TMA => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            TAC => IOReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
            IF => IOReg::new(0xE1, 0b00011111, 0b00011111, WriteBehavior::Standard),
            NR10 => IOReg::new(0x80, 0b01111111, 0b01111111, WriteBehavior::Standard),
            NR11 => IOReg::new(0xBF, 0b11000000, 0b11111111, WriteBehavior::Standard),
            NR12 => IOReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR13 => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR14 => IOReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR21 => IOReg::new(0x3F, 0b11000000, 0b11111111, WriteBehavior::Standard),
            NR22 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR23 => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR24 => IOReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR30 => IOReg::new(0x7F, 0b10000000, 0b10000000, WriteBehavior::Standard),
            NR31 => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            NR32 => IOReg::new(0x9F, 0b01100000, 0b01100000, WriteBehavior::Standard),
            NR33 => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR34 => IOReg::new(0xBF, 0b01000000, 0b11000111, WriteBehavior::Standard),
            NR41 => IOReg::new(0xFF, 0b00000000, 0b00111111, WriteBehavior::Standard),
            NR42 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR43 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR44 => IOReg::new(0xBF, 0b01000000, 0b11000000, WriteBehavior::Standard),
            NR50 => IOReg::new(0x77, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR51 => IOReg::new(0xF3, 0b11111111, 0b11111111, WriteBehavior::Standard),
            NR52 => IOReg::new(match model {
                Model::GameBoy(_) => 0xF1,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0xF0,
            }, 0b10001111, 0b00001111, WriteBehavior::Standard),
            WAV00 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV01 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV02 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV03 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV04 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV05 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV06 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV07 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV08 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV09 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV10 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV11 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV12 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV13 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV14 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WAV15 => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            LCDC => IOReg::new(0x91, 0b11111111, 0b11111111, WriteBehavior::Standard),
            STAT => IOReg::new(match model {
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x81,
                Model::GameBoy(_) => 0x85,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b01111111, 0b01111000, WriteBehavior::Standard),
            SCY => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            SCX => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            LY => IOReg::new(match model {
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x91,
                Model::GameBoy(_) => 0x00,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 0b11111111, 0b00000000, WriteBehavior::Standard),
            LYC => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            DMA => IOReg::new(match model {
                Model::GameBoy(_) | Model::SuperGameBoy(_) => 0xFF,
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0x00,
            }, 0b11111111, 0b11111111, WriteBehavior::Standard),
            BGP => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new(0xFC, 0b11111111, 0b11111111, WriteBehavior::Standard),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new_unimplemented(),
            },
            OBP0 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new_unimplemented(),
            },
            OBP1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // Unitialized, but 0xFF is a common value
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new_unimplemented(),
            },
            WY => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            WX => IOReg::new(0x00, 0b11111111, 0b11111111, WriteBehavior::Standard),
            KEY0 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(rand::random(), 0b00000100, 0b00000000, WriteBehavior::Standard), // TODO: Value is supposed to be based on header contents, Allow writing during boot ROM if included
            }, 
            KEY1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0x7E, 0b10000001, 0b00000001, WriteBehavior::Standard),
            }, 
            VBK => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFE, 0b00000001, 0b00000001, WriteBehavior::Standard),
            }, 
            BOOT => IOReg::new(0xFF, 0b00000000, 0b00000001, WriteBehavior::UnmapBootRom), // TODO: Verify write behavior
            HDMA1 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard),
            }, 
            HDMA2 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
            }, 
            HDMA3 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure upper three bits are ignored
            }, 
            HDMA4 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b00000000, 0b11111111, WriteBehavior::Standard), // TODO: Ensure lower four bits are ignored
            }, 
            HDMA5 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Ensure proper read behavior
            }, 
            RP => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0x3E, 0b11000011, 0b00000010, WriteBehavior::Standard),
            },
            BCPS => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
            },
            BCPD => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
            },
            OCPS => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b10111111, 0b10111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles
            },
            OCPD => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b11111111, 0b11111111, WriteBehavior::Standard), // TODO: Value is supposed to be based on boot rom cycles, change write mask depending on addressed palette entry
            },
            OPRI => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b00000001, 0b00000001, WriteBehavior::Standard), // TODO: Verify startup value and masks
            },
            SVBK => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xF8, 0b00000111, 0b00000111, WriteBehavior::Standard),
            },
            PCM12 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
            },
            PCM34 => match model { 
                Model::GameBoy(_) | Model::SuperGameBoy(_) => IOReg::new_unimplemented(),
                Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => IOReg::new(0xFF, 0b11111111, 0b00000000, WriteBehavior::Standard), // TODO: Verify startup value
            },
            IE => IOReg::new(0x00, 0b00011111, 0b00011111, WriteBehavior::Standard),
            _ => IOReg::new_unimplemented()
        })}
    }
}

impl Index<usize> for IOMap {
    type Output = Rc<IOReg>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}
impl IndexMut<usize> for IOMap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}

pub struct IOReg {
    value: Cell<u8>,
    read_mask: Cell<u8>,
    write_mask: Cell<u8>,
    write_fn: fn(&IOReg, val: u8)
}

impl IOReg {
    pub(self) fn new(value: u8, read_mask: u8, write_mask: u8, reg_type: WriteBehavior) -> Rc<Self> {
        let write_fn = match reg_type {
            WriteBehavior::Standard => IOReg::write_standard,
            WriteBehavior::ResetOnWrite => IOReg::write_reset,
            WriteBehavior::UnmapBootRom => IOReg::write_boot,
        };

        Rc::new(IOReg { value: Cell::new(value), read_mask: Cell::new(read_mask), write_mask: Cell::new(write_mask), write_fn})
    }

    pub(self) fn new_unimplemented() -> Rc<Self> {
        IOReg::new(0xFF, 0x00, 0x00, WriteBehavior::Standard)
    }

    /// Returns a copy of the contained value.
    pub fn get(&self) -> u8 {
        self.value.get()
    }

    /// Sets the contained value.
    pub fn set(&self, val: u8) {
        self.value.set(val)
    }

    /// Returns a copy of the contained value, with write-only
    /// and unimplemented bits replaced by a 1.
    pub fn read(&self) -> u8 {
        self.value.get() | !self.read_mask.get()
    }

    /// Sets the contained value, with read-only
    /// and unimplemented bits ignored.
    pub fn write(&self, val: u8) {
        (self.write_fn)(self, val)
    }

    fn write_standard(&self, val: u8) {
        let write_mask = self.write_mask.get();
        self.value.set((self.value.get() & !write_mask) | (val & write_mask))
    }

    fn write_reset(&self, _val: u8) {
        self.value.set(0x00);
    }

    fn write_boot(&self, _val: u8) {
        todo!("BANK register write behavior is not yet implemented")
    }

    /// Redefines which bits of this register are
    /// readable and/or writable. 
    pub fn change_masks(&self, read_mask: u8, write_mask: u8) {
        self.read_mask.set(read_mask);
        self.write_mask.set(write_mask);
    }
}

enum WriteBehavior {
    Standard,
    ResetOnWrite,
    UnmapBootRom
}