use std::{cell::RefCell, rc::Rc};

use crate::{common::bit::BitVec, deserialize, gameboy::{GBRevision, Interrupt, InterruptFlags, Model, memory::{MMIO, MemoryMappedIo}, ppu::PpuMode}, serialize};

pub struct PpuState {
    ppu_mode: Rc<RefCell<PpuMode>>,
    
    dots: u32,
    ly: u8,
    lyc: u8,

    stat_interrupt_select: u8,
    interrupt_flags: Rc<RefCell<InterruptFlags>>
}

impl PpuState {
    pub fn new(model: &Rc<Model>, ppu_mode: Rc<RefCell<PpuMode>>, interrupt_flags: Rc<RefCell<InterruptFlags>>) -> Self {
        PpuState { 
            ppu_mode, 
            ly: match **model {
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x91,
                Model::GameBoy(_) => 0x00,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
            }, 
            dots: 0,
            lyc: 0,
            stat_interrupt_select: 0,

            interrupt_flags
        }
    }

    const DOTS_PER_SCANLINE: u32 = 456;
    const DOTS_PER_FRAME: u32 = 70224;

    pub fn tick(&mut self) {
        self.dots = (self.dots + 1) % Self::DOTS_PER_FRAME;
        let ly = (self.dots / Self::DOTS_PER_SCANLINE) as u8;
        self.ly_eq_lyc_check(ly == self.lyc);
        self.ly = ly;
    }

    fn ly_eq_lyc_check(&mut self, new_ly_eq_lyc: bool) {
        // Detect rising edge on stat interrupt line
        if (new_ly_eq_lyc && self.ly != self.lyc && self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.borrow().as_stat_line_flag())) {
            self.interrupt_flags.borrow_mut().request(Interrupt::Stat);
        }
    }

    pub fn get_mode(&self) -> PpuMode {
        *self.ppu_mode.borrow()
    }

    pub fn set_mode(&mut self, ppu_mode: PpuMode) {
        if ppu_mode == PpuMode::VBlank {
            self.interrupt_flags.borrow_mut().request(Interrupt::Vblank);
        }

        // Detect rising edge on stat interrupt line
        if (self.ly != self.lyc || !self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.borrow().as_stat_line_flag()))
        && (self.stat_interrupt_select.test_bits(ppu_mode.as_stat_line_flag())) {
            self.interrupt_flags.borrow_mut().request(Interrupt::Stat);
        }
        
        self.ppu_mode.replace(ppu_mode);
    }

    pub fn get_dots(&self) -> u32 {
        self.dots
    }

    pub fn get_dot_coords(&self) -> (u8, u8) {
        ((self.dots % Self::DOTS_PER_SCANLINE) as u8, self.ly)
    }
}

impl MemoryMappedIo<{MMIO::STAT as u16}> for PpuState {
    fn read(&self) -> u8 {
        serialize!(
            0b10000000;
            (self.stat_interrupt_select) => 6..=3;
            ((self.ly == self.lyc) as u8) =>> 2;
            (*self.ppu_mode.borrow() as u8) =>> 1..=0;
        )
    }
    
    fn write(&mut self, val: u8) {
        deserialize!(val;
            6..=3 => (self.stat_interrupt_select);
        );
    }
}

impl MemoryMappedIo<{MMIO::LY as u16}> for PpuState {
    fn read(&self) -> u8 {
        self.ly
    }
    
    fn write(&mut self, _val: u8) {
        // Do nothing -- readonly
    }
}

impl MemoryMappedIo<{MMIO::LYC as u16}> for PpuState {
    fn read(&self) -> u8 {
        self.lyc
    }
    
    fn write(&mut self, lyc: u8) {
        self.ly_eq_lyc_check(self.ly == lyc);
        self.lyc = lyc;
    }
}