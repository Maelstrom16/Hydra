use std::rc::Rc;

use crate::gameboy::{Model, memory::io::{self, IOMap, IOReg}};

pub struct Wram {
    wram: Box<[[u8; 0x1000]]>,
    wbk: Rc<IOReg>,
}

impl Wram {
    pub fn new(model: Model, io: &IOMap) -> Self {
        let mut result = Wram {
            wram: Box::new([[0; 0x1000]; 2]),
            wbk: io[io::SVBK].clone()
        };
        result.change_model(model);

        result
    }

    pub fn change_model(&mut self, model: Model) {
        match (self.is_monochrome(), model.is_monochrome()) {
            (false, true) => self.wram = Box::from(&self.wram[0..2]),
            (true, false) => self.wram = [&self.wram[0..2], &[[0; 0x1000]; 6]].concat().into_boxed_slice(),
            _ => {}
        }
    }

    pub fn read_u8(&self, local_address: usize) -> u8 {
        self.wram[self.get_bank_id(local_address) as usize][local_address % 0x1000]
    }

    pub fn write_u8(&mut self, value: u8, local_address: usize) {
        self.wram[self.get_bank_id(local_address) as usize][local_address % 0x1000] = value
    }

    fn is_monochrome(&self) -> bool {
        let bank_count = self.wram.len();

        bank_count == 2
    }

    fn is_color(&self) -> bool {
        let bank_count = self.wram.len();

        bank_count == 8
    }

    fn get_bank_id(&self, address: usize) -> u8 {
        match address {
            0..0x1000 => 0,
            _ => if self.is_monochrome() {1} else {(self.wbk.get() & 0b00000111).max(1)}
        }
    }
}