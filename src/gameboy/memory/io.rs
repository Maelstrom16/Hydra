use std::{cell::Cell, rc::Rc};

use crate::gameboy::{GBRevision, Model};

// I/O Register Addresses
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

pub struct IO {
    pub p1: Rc<Cell<u8>>,
    pub sb: Rc<Cell<u8>>,
    pub sc: Rc<Cell<u8>>,
    pub div: Rc<Cell<u8>>,
    pub tima: Rc<Cell<u8>>,
    pub tma: Rc<Cell<u8>>,
    pub tac: Rc<Cell<u8>>,
    pub r#if: Rc<Cell<u8>>,
    pub nr10: Rc<Cell<u8>>,
    pub nr11: Rc<Cell<u8>>,
    pub nr12: Rc<Cell<u8>>,
    pub nr13: Rc<Cell<u8>>,
    pub nr14: Rc<Cell<u8>>,
    pub nr21: Rc<Cell<u8>>,
    pub nr22: Rc<Cell<u8>>,
    pub nr23: Rc<Cell<u8>>,
    pub nr24: Rc<Cell<u8>>,
    pub nr30: Rc<Cell<u8>>,
    pub nr31: Rc<Cell<u8>>,
    pub nr32: Rc<Cell<u8>>,
    pub nr33: Rc<Cell<u8>>,
    pub nr34: Rc<Cell<u8>>,
    pub nr41: Rc<Cell<u8>>,
    pub nr42: Rc<Cell<u8>>,
    pub nr43: Rc<Cell<u8>>,
    pub nr44: Rc<Cell<u8>>,
    pub nr50: Rc<Cell<u8>>,
    pub nr51: Rc<Cell<u8>>,
    pub nr52: Rc<Cell<u8>>,
    pub wav00: Rc<Cell<u8>>,
    pub wav01: Rc<Cell<u8>>,
    pub wav02: Rc<Cell<u8>>,
    pub wav03: Rc<Cell<u8>>,
    pub wav04: Rc<Cell<u8>>,
    pub wav05: Rc<Cell<u8>>,
    pub wav06: Rc<Cell<u8>>,
    pub wav07: Rc<Cell<u8>>,
    pub wav08: Rc<Cell<u8>>,
    pub wav09: Rc<Cell<u8>>,
    pub wav10: Rc<Cell<u8>>,
    pub wav11: Rc<Cell<u8>>,
    pub wav12: Rc<Cell<u8>>,
    pub wav13: Rc<Cell<u8>>,
    pub wav14: Rc<Cell<u8>>,
    pub wav15: Rc<Cell<u8>>,
    pub lcdc: Rc<Cell<u8>>,
    pub stat: Rc<Cell<u8>>,
    pub scy: Rc<Cell<u8>>,
    pub scx: Rc<Cell<u8>>,
    pub ly: Rc<Cell<u8>>,
    pub lyc: Rc<Cell<u8>>,
    pub dma: Rc<Cell<u8>>,
    pub bgp: Rc<Cell<u8>>,
    pub obp0: Rc<Cell<u8>>,
    pub obp1: Rc<Cell<u8>>,
    pub wy: Rc<Cell<u8>>,
    pub wx: Rc<Cell<u8>>,
    pub key0: Rc<Cell<u8>>,
    pub key1: Rc<Cell<u8>>,
    pub vbk: Rc<Cell<u8>>,
    pub boot: Rc<Cell<u8>>,
    pub hdma1: Rc<Cell<u8>>,
    pub hdma2: Rc<Cell<u8>>,
    pub hdma3: Rc<Cell<u8>>,
    pub hdma4: Rc<Cell<u8>>,
    pub hdma5: Rc<Cell<u8>>,
    pub rp: Rc<Cell<u8>>,
    pub bcps: Rc<Cell<u8>>,
    pub bpcd: Rc<Cell<u8>>,
    pub ocps: Rc<Cell<u8>>,
    pub ocpd: Rc<Cell<u8>>,
    pub opri: Rc<Cell<u8>>,
    pub svbk: Rc<Cell<u8>>,
    pub pcm12: Rc<Cell<u8>>,
    pub pcm34: Rc<Cell<u8>>,
    pub ie: Rc<Cell<u8>>,
}

impl IO {
    pub fn new(model: Model) -> Self {
        // Define default values for all variables (DMG/MGB)
        let p1 = Rc::new(Cell::new(0xCF));
        let sb = Rc::new(Cell::new(0x00));
        let sc = Rc::new(Cell::new(0x7E));
        let div = Rc::new(Cell::new(0xAB));
        let tima = Rc::new(Cell::new(0x00));
        let tma = Rc::new(Cell::new(0x00));
        let tac = Rc::new(Cell::new(0xF8));
        let r#if = Rc::new(Cell::new(0xE1));
        let nr10 = Rc::new(Cell::new(0x80));
        let nr11 = Rc::new(Cell::new(0xBF));
        let nr12 = Rc::new(Cell::new(0xF3));
        let nr13 = Rc::new(Cell::new(0xFF));
        let nr14 = Rc::new(Cell::new(0xBF));
        let nr21 = Rc::new(Cell::new(0x3F));
        let nr22 = Rc::new(Cell::new(0x00));
        let nr23 = Rc::new(Cell::new(0xFF));
        let nr24 = Rc::new(Cell::new(0xBF));
        let nr30 = Rc::new(Cell::new(0x7F));
        let nr31 = Rc::new(Cell::new(0xFF));
        let nr32 = Rc::new(Cell::new(0x9F));
        let nr33 = Rc::new(Cell::new(0xFF));
        let nr34 = Rc::new(Cell::new(0xBF));
        let nr41 = Rc::new(Cell::new(0xFF));
        let nr42 = Rc::new(Cell::new(0x00));
        let nr43 = Rc::new(Cell::new(0x00));
        let nr44 = Rc::new(Cell::new(0xBF));
        let nr50 = Rc::new(Cell::new(0x77));
        let nr51 = Rc::new(Cell::new(0xF3));
        let nr52 = Rc::new(Cell::new(0xF1));
        let wav00 = Rc::new(Cell::new(0x00));
        let wav01 = Rc::new(Cell::new(0x00));
        let wav02 = Rc::new(Cell::new(0x00));
        let wav03 = Rc::new(Cell::new(0x00));
        let wav04 = Rc::new(Cell::new(0x00));
        let wav05 = Rc::new(Cell::new(0x00));
        let wav06 = Rc::new(Cell::new(0x00));
        let wav07 = Rc::new(Cell::new(0x00));
        let wav08 = Rc::new(Cell::new(0x00));
        let wav09 = Rc::new(Cell::new(0x00));
        let wav10 = Rc::new(Cell::new(0x00));
        let wav11 = Rc::new(Cell::new(0x00));
        let wav12 = Rc::new(Cell::new(0x00));
        let wav13 = Rc::new(Cell::new(0x00));
        let wav14 = Rc::new(Cell::new(0x00));
        let wav15 = Rc::new(Cell::new(0x00));
        let lcdc = Rc::new(Cell::new(0x91));
        let stat = Rc::new(Cell::new(0x85));
        let scy = Rc::new(Cell::new(0x00));
        let scx = Rc::new(Cell::new(0x00));
        let ly = Rc::new(Cell::new(0x00));
        let lyc = Rc::new(Cell::new(0x00));
        let dma = Rc::new(Cell::new(0xFF));
        let bgp = Rc::new(Cell::new(0xFC));
        let obp0 = Rc::new(Cell::new(0xFF)); // Unitialized, but 0xFF is a common value
        let obp1 = Rc::new(Cell::new(0xFF)); // Unitialized, but 0xFF is a common value
        let wy = Rc::new(Cell::new(0x00));
        let wx = Rc::new(Cell::new(0x00));
        let key0 = Rc::new(Cell::new(0xFF)); // Unitialized, but 0xFF is a common value
        let key1 = Rc::new(Cell::new(0xFF));
        let vbk = Rc::new(Cell::new(0xFF));
        let boot = Rc::new(Cell::new(0xFF));
        let hdma1 = Rc::new(Cell::new(0xFF));
        let hdma2 = Rc::new(Cell::new(0xFF));
        let hdma3 = Rc::new(Cell::new(0xFF));
        let hdma4 = Rc::new(Cell::new(0xFF));
        let hdma5 = Rc::new(Cell::new(0xFF));
        let rp = Rc::new(Cell::new(0xFF));
        let bcps = Rc::new(Cell::new(0xFF));
        let bpcd = Rc::new(Cell::new(0xFF));
        let ocps = Rc::new(Cell::new(0xFF));
        let ocpd = Rc::new(Cell::new(0xFF));
        let opri = Rc::new(Cell::new(0xFF));
        let svbk = Rc::new(Cell::new(0xFF));
        let pcm12 = Rc::new(Cell::new(0xFF));
        let pcm34 = Rc::new(Cell::new(0xFF));
        let ie = Rc::new(Cell::new(0x00));

        // Define model-specific registers
        match model {
            Model::GameBoy(Some(GBRevision::DMG0)) => {
                div.set(0x18);
                stat.set(0x81);
                ly.set(0x91);
            }
            Model::GameBoy(_) => {
                // All default values
            }
            Model::SuperGameBoy(_) => {
                div.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                nr52.set(0xF0);
                stat.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                ly.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
            }
            Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => {
                sc.set(0x7F);
                div.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                nr52.set(0xF0);
                stat.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                ly.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
                dma.set(0x00);
                key0.set(rand::random()); // TODO: Number is supposed to be based on boot rom cycles
            }
        }
        
        // Build IO object
        IO { p1, sb, sc, div, tima, tma, tac, r#if, nr10, nr11, nr12, nr13, nr14, nr21, nr22, nr23, nr24, nr30, nr31, nr32, nr33, nr34, nr41, nr42, nr43, nr44, nr50, nr51, nr52, wav00, wav01, wav02, wav03, wav04, wav05, wav06, wav07, wav08, wav09, wav10, wav11, wav12, wav13, wav14, wav15, lcdc, stat, scy, scx, ly, lyc, dma, bgp, obp0, obp1, wy, wx, key0, key1, vbk, boot, hdma1, hdma2, hdma3, hdma4, hdma5, rp, bcps, bpcd, ocps, ocpd, opri, svbk, pcm12, pcm34, ie }
    }
}