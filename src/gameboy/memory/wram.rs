use std::{cell::Cell, rc::Rc};

use crate::{gameboy::{Model, memory::io::{self, IO}}};

pub struct Wram {
    wram: Box<[[u8; 0x2000]]>,
    wbk: Rc<Cell<u8>>,
}

impl Wram {
    pub fn new(model: Model, io: &IO) -> Self {
        let mut result = Wram {
            wram: Box::new([[0; 0x2000]; 1]),
            wbk: io[io::SVBK].clone()
        };
        result.change_model(model);

        result
    }

    pub fn change_model(&mut self, model: Model) {
        let bank_count = self.wram.len();
        match (bank_count, model) {
            (2, Model::GameBoy(_) | Model::SuperGameBoy(_)) => self.wram = Box::new([self.wram[0]]),
            (1, Model::GameBoyColor(_) | Model::GameBoyAdvance(_)) => self.wram = [[self.wram[0]].as_slice(), [[0; 0x2000]; 7].as_slice()].concat().into_boxed_slice(),
            _ => {}
        }
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        self.wram[self.wbk.get() as usize][address]
    }

    pub fn write_u8(&mut self, value: u8, address: usize) {
        self.wram[self.wbk.get() as usize][address] = value
    }
}