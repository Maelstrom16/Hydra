mod deserialized;

use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::gameboy::Model;
use crate::gameboy::memory::io::deserialized::{RegStat, RegVbk};
use crate::gameboy::memory::io::{self, GBReg, IOMap};

pub const ADDRESS_OFFSET: u16 = 0x8000;

pub struct Vram {
    vram: Box<[[u8; 0x2000]]>,
    vbk: Rc<GBReg>,
    stat: Rc<GBReg>,
}

impl Vram {
    pub fn new(model: Model, io: &IOMap) -> Self {
        let mut result = Vram {
            vram: Box::new([[0; 0x2000]; 1]),
            vbk: io.clone_pointer(io::MMIO::VBK),
            stat: io.clone_pointer(io::MMIO::STAT),
        };
        result.change_model(model);

        result
    }

    pub fn change_model(&mut self, model: Model) {
        match (self.is_monochrome(), model.is_monochrome()) {
            (false, true) => self.vram = Box::from(&self.vram[0..1]),
            (true, false) => self.vram = Box::new([self.vram[0], [0; 0x2000]]),
            _ => {}
        }
    }

    pub fn read_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.is_accessible() {
            Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)])
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.is_accessible() {
            Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)] = value)
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }

    pub fn unbound_read_u8(&self, address: u16, bank: u8) -> u8 {
        self.vram[bank as usize][Vram::localize_address(address)]
    }

    fn is_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3
        (self.stat.get() & 0b00000011) != 3
    }

    fn is_monochrome(&self) -> bool {
        let bank_count = self.vram.len();

        bank_count == 1
    }

    fn is_color(&self) -> bool {
        let bank_count = self.vram.len();

        bank_count == 2
    }

    fn get_bank_id(&self) -> u8 {
        if self.is_monochrome() {0} else {self.vbk.get() & 0b00000001}
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}