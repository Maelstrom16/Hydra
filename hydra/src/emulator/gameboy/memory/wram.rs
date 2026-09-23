use std::rc::Rc;

use crate::{deserialize, emulator::gameboy::GbModel, serialize};

pub const ADDRESS_OFFSET: u16 = 0xC000;

pub struct Wram {
    wram: Box<[[u8; 0x1000]]>,
    wbk: u8,
}

impl Wram {
    pub fn new<M: GbModel>() -> Self {
        let bank_count = match M::is_monochrome() {
            true => 2,
            false => 8
        };

        Wram {
            wram: vec![[0; 0x1000]; bank_count].into_boxed_slice(),
            wbk: 0
        }
    }

    pub fn read_u8(&self, address: u16, cgb_mode: bool) -> u8 {
        let local_address = Wram::localize_address(address);

        self.wram[self.get_bank_id(local_address, cgb_mode) as usize][local_address % 0x1000]
    }

    pub fn write_u8(&mut self, value: u8, address: u16, cgb_mode: bool) {
        let local_address = Wram::localize_address(address);

        self.wram[self.get_bank_id(local_address, cgb_mode) as usize][local_address % 0x1000] = value
    }

    fn get_bank_id(&self, address: usize, cgb_mode: bool) -> u8 {
        match address {
            0..0x1000 => 0,
            _ => if cgb_mode {self.wbk.max(1)} else {1}
        }
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}

impl Wram {
    pub fn read_wbk(&self) -> u8 {
        serialize!(
            0b11111000;
            (self.wbk) =>> [2..=0];
        )
    }

    pub fn write_wbk(&mut self, val: u8) {
        deserialize!(val;
            [2..=0] =>> (self.wbk);
        );
    }
}