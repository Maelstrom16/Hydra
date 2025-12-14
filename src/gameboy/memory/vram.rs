use std::cell::Cell;
use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::gameboy::Model;
use crate::gameboy::memory::io::{self, IO};

pub struct Vram {
    vram: Box<[[u8; 0x2000]]>,
    vbk: Rc<Cell<u8>>,
    stat: Rc<Cell<u8>>,
}

impl Vram {
    pub fn new(model: Model, io: &IO) -> Self {
        let mut result = Vram {
            vram: Box::new([[0; 0x2000]; 1]),
            vbk: io[io::VBK].clone(),
            stat: io[io::STAT].clone()
        };
        result.change_model(model);

        result
    }

    pub fn change_model(&mut self, model: Model) {
        let bank_count = self.vram.len();
        match (bank_count, model) {
            (2, Model::GameBoy(_) | Model::SuperGameBoy(_)) => self.vram = Box::new([self.vram[0]]),
            (1, Model::GameBoyColor(_) | Model::GameBoyAdvance(_)) => self.vram = Box::new([self.vram[0], [0; 0x2000]]),
            _ => {}
        }
    }

    pub fn read_u8(&self, address: usize) -> Result<u8, HydraIOError> {
        if self.is_accessible() {
            Ok(self.vram[self.vbk.get() as usize][address])
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }

    pub fn write_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError> {
        if self.is_accessible() {
            Ok(self.vram[self.vbk.get() as usize][address] = value)
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }

    pub fn unbound_read_u8(&self, address: usize, bank: u8) -> u8 {
        self.vram[bank as usize][address]
    }

    fn is_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3
        (self.stat.get() & 0b00000011) != 3
    }
}