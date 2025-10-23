use std::{array, cell::Cell, ops::{Index, IndexMut}, rc::Rc};

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
pub const BPCD: usize = 0xFF69 - ADDRESS_OFFSET;
pub const OCPS: usize = 0xFF6A - ADDRESS_OFFSET;
pub const OCPD: usize = 0xFF6B - ADDRESS_OFFSET;
pub const OPRI: usize = 0xFF6C - ADDRESS_OFFSET;
pub const SVBK: usize = 0xFF70 - ADDRESS_OFFSET;
pub const PCM12: usize = 0xFF76 - ADDRESS_OFFSET;
pub const PCM34: usize = 0xFF77 - ADDRESS_OFFSET;
pub const IE: usize = 0xFFFF - ADDRESS_OFFSET - 0x7F; // To compensate for HRAM

pub struct IO {
    registers: [Rc<Cell<u8>>; 0x80 + 1]
}

impl IO {
    pub fn new(model: Model) -> Self {
        let registers = array::from_fn(|_| Rc::new(Cell::new(0x00)));
        // Define default values for all variables (DMG/MGB)
        registers[P1].set(0xCF);
        registers[SB].set(0x00);
        registers[SC].set(0x7E);
        registers[DIV].set(0xAB);
        registers[TIMA].set(0x00);
        registers[TMA].set(0x00);
        registers[TAC].set(0xF8);
        registers[IF].set(0xE1);
        registers[NR10].set(0x80);
        registers[NR11].set(0xBF);
        registers[NR12].set(0xF3);
        registers[NR13].set(0xFF);
        registers[NR14].set(0xBF);
        registers[NR21].set(0x3F);
        registers[NR22].set(0x00);
        registers[NR23].set(0xFF);
        registers[NR24].set(0xBF);
        registers[NR30].set(0x7F);
        registers[NR31].set(0xFF);
        registers[NR32].set(0x9F);
        registers[NR33].set(0xFF);
        registers[NR34].set(0xBF);
        registers[NR41].set(0xFF);
        registers[NR42].set(0x00);
        registers[NR43].set(0x00);
        registers[NR44].set(0xBF);
        registers[NR50].set(0x77);
        registers[NR51].set(0xF3);
        registers[NR52].set(0xF1);
        registers[WAV00].set(0x00);
        registers[WAV01].set(0x00);
        registers[WAV02].set(0x00);
        registers[WAV03].set(0x00);
        registers[WAV04].set(0x00);
        registers[WAV05].set(0x00);
        registers[WAV06].set(0x00);
        registers[WAV07].set(0x00);
        registers[WAV08].set(0x00);
        registers[WAV09].set(0x00);
        registers[WAV10].set(0x00);
        registers[WAV11].set(0x00);
        registers[WAV12].set(0x00);
        registers[WAV13].set(0x00);
        registers[WAV14].set(0x00);
        registers[WAV15].set(0x00);
        registers[LCDC].set(0x91);
        registers[STAT].set(0x85);
        registers[SCY].set(0x00);
        registers[SCX].set(0x00);
        registers[LY].set(0x00);
        registers[LYC].set(0x00);
        registers[DMA].set(0xFF);
        registers[BGP].set(0xFC);
        registers[OBP0].set(0xFF); // Unitialized, but 0xFF is a common value
        registers[OBP1].set(0xFF); // Unitialized, but 0xFF is a common value
        registers[WY].set(0x00);
        registers[WX].set(0x00);
        registers[KEY0].set(0xFF); // Unitialized, but 0xFF is a common value
        registers[KEY1].set(0xFF);
        registers[VBK].set(0xFF);
        registers[BOOT].set(0xFF);
        registers[HDMA1].set(0xFF);
        registers[HDMA2].set(0xFF);
        registers[HDMA3].set(0xFF);
        registers[HDMA4].set(0xFF);
        registers[HDMA5].set(0xFF);
        registers[RP].set(0xFF);
        registers[BCPS].set(0xFF);
        registers[BPCD].set(0xFF);
        registers[OCPS].set(0xFF);
        registers[OCPD].set(0xFF);
        registers[OPRI].set(0xFF);
        registers[SVBK].set(0xFF);
        registers[PCM12].set(0xFF);
        registers[PCM34].set(0xFF);
        registers[IE].set(0x00);

        // Define model-specific registers
        match model {
            Model::GameBoy(Some(GBRevision::DMG0)) => {
                registers[DIV].set(0x18);
                registers[STAT].set(0x81);
                registers[LY].set(0x91);
            }
            Model::GameBoy(_) => {
                // All default values
            }
            Model::SuperGameBoy(_) => {
                registers[DIV].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                registers[NR52].set(0xF0);
                registers[STAT].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                registers[LY].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
            }
            Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => {
                registers[SC].set(0x7F);
                registers[DIV].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                registers[NR52].set(0xF0);
                registers[STAT].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                registers[LY].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                registers[DMA].set(0x00);
                registers[KEY0].set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
            }
        }
        
        // Build IO object
        IO { registers }
    }
}

impl Index<usize> for IO {
    type Output = Rc<Cell<u8>>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}
impl IndexMut<usize> for IO {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}