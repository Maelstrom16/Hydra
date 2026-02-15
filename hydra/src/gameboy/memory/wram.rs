use std::rc::Rc;

use crate::gameboy::Model;

pub const ADDRESS_OFFSET: u16 = 0xC000;

pub struct Wram {
    model: Rc<Model>,
    wram: Box<[[u8; 0x1000]]>,
    wbk: u8,
}

impl Wram {
    pub fn new(model: Rc<Model>) -> Self {
        let bank_count = match model.is_monochrome() {
            true => 2,
            false => 8
        };

        Wram {
            model,
            wram: vec![[0; 0x1000]; 2].into_boxed_slice(),
            wbk: 0
        }
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        let local_address = Wram::localize_address(address);

        self.wram[self.get_bank_id(local_address) as usize][local_address % 0x1000]
    }

    pub fn write_u8(&mut self, value: u8, address: u16) {
        let local_address = Wram::localize_address(address);

        self.wram[self.get_bank_id(local_address) as usize][local_address % 0x1000] = value
    }

    fn get_bank_id(&self, address: usize) -> u8 {
        match address {
            0..0x1000 => 0,
            _ => if self.model.is_monochrome() {1} else {self.wbk.max(1)}
        }
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}