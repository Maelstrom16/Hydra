use std::{cell::RefCell, rc::Rc};

use crate::{common::bit::BitVec, deserialize, gameboy::{GBRevision, Model, interrupt::{Interrupt, InterruptFlags}, memory::MemoryMap, ppu::PpuMode}, serialize};

pub struct PpuState {
    ppu_mode: PpuMode,
    
    dots: u32,
    ly: u8,
    lyc: u8,

    stat_interrupt_select: u8,
}

impl PpuState {
    pub fn new(model: &Rc<Model>) -> Self {
        PpuState { 
            ppu_mode: PpuMode::OAMScan,
            ly: match **model {
                Model::GameBoy(GBRevision::DMG0) => 0x91,
                Model::GameBoy(_) => 0x00,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 
            dots: 0,
            lyc: 0,
            stat_interrupt_select: 0,
        }
    }

    const DOTS_PER_SCANLINE: u32 = 456;
    const DOTS_PER_FRAME: u32 = 70224;

    pub fn tick(&mut self, interrupt_flags: &mut InterruptFlags) {
        self.dots = (self.dots + 1) % Self::DOTS_PER_FRAME;
        let ly = (self.dots / Self::DOTS_PER_SCANLINE) as u8;
        self.ly_eq_lyc_check(ly == self.lyc, interrupt_flags);
        self.ly = ly;
    }

    fn ly_eq_lyc_check(&mut self, new_ly_eq_lyc: bool, interrupt_flags: &mut InterruptFlags) {
        // Detect rising edge on stat interrupt line
        if (new_ly_eq_lyc && self.ly != self.lyc && self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.as_stat_line_flag())) {
            interrupt_flags.request(Interrupt::Stat);
        }
    }

    pub fn get_mode(&self) -> PpuMode {
        self.ppu_mode
    }

    pub fn set_mode(&mut self, ppu_mode: PpuMode, interrupt_flags: &mut InterruptFlags) {
        if ppu_mode == PpuMode::VBlank {
            interrupt_flags.request(Interrupt::Vblank);
        }

        // Detect rising edge on stat interrupt line
        if (self.ly != self.lyc || !self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.as_stat_line_flag()))
        && (self.stat_interrupt_select.test_bits(ppu_mode.as_stat_line_flag())) {
            interrupt_flags.request(Interrupt::Stat);
        }
        
        self.ppu_mode = ppu_mode;
    }

    pub fn get_dots(&self) -> u32 {
        self.dots
    }

    pub fn get_dot_coords(&self) -> (u8, u8) {
        ((self.dots % Self::DOTS_PER_SCANLINE) as u8, self.ly)
    }
}

impl PpuState {
    pub fn read_stat(&self) -> u8 {
        serialize!(
            0b10000000;
            (self.stat_interrupt_select) => 6..=3;
            ((self.ly == self.lyc) as u8) =>> 2;
            (self.ppu_mode as u8) =>> 1..=0;
        )
    }
    
    pub fn write_stat(&mut self, val: u8) {
        deserialize!(val;
            6..=3 => (self.stat_interrupt_select);
        );
    }
    
    pub fn read_ly(&self) -> u8 {
        self.ly
    }
    
    pub fn write_ly(&mut self, _val: u8) {
        // Do nothing -- readonly
    }
    
    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }
    
    pub fn write_lyc(&mut self, lyc: u8, interrupt_flags: &mut InterruptFlags) {
        self.ly_eq_lyc_check(self.ly == lyc, interrupt_flags);
        self.lyc = lyc;
    }
}