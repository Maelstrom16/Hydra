use std::cell::RefCell;
use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::emulator::gameboy::GbModel;
use crate::emulator::gameboy::ppu::PpuMode;
use crate::emulator::gameboy::ppu::attributes::TileAttributes;
use crate::{deserialize, serialize};

pub const ADDRESS_OFFSET: u16 = 0x8000;

pub struct Vram {
    vram: Box<[[u8; 0x2000]]>,
    vbk: u8,
}

impl Vram {
    pub fn new<M: GbModel>() -> Self {
        let bank_count = match M::is_monochrome() {
            true => 1,
            false => 2
        };

        Vram {
            vram: vec![[0; 0x2000]; bank_count].into_boxed_slice(),
            vbk: 0,
        }
    }

    pub fn read_u8(&self, address: u16, cgb_mode: bool) -> Result<u8, HydraIOError> {
        Ok(self.vram[self.get_bank_id(cgb_mode) as usize][Vram::localize_address(address)])
    }

    pub fn write_u8(&mut self, value: u8, address: u16, cgb_mode: bool) -> Result<(), HydraIOError> {
        Ok(self.vram[self.get_bank_id(cgb_mode) as usize][Vram::localize_address(address)] = value)
    }

    pub fn read_tile_data(&self, address: u16, bank: u8) -> u8 {
        self.vram[bank as usize][Vram::localize_address(address)]
    }

    pub fn read_tile_map(&self, address: u16, cgb_mode: bool) -> (u8, TileAttributes) {
        let address = Vram::localize_address(address);
        (self.vram[0][address], match cgb_mode {
            true => TileAttributes::from_u8(self.vram[1][address], cgb_mode),
            false => TileAttributes::default(),
        }) 
    }

    fn get_bank_id(&self, cgb_mode: bool) -> u8 {
        if cgb_mode {self.vbk} else {0}
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}

impl Vram {
    pub fn read_vbk(&self) -> u8 {
        serialize!(
            0b11111110;
            (self.vbk) =>> [0];
        )
    }

    pub fn write_vbk(&mut self, val: u8) {
        deserialize!(val;
            [0] =>> (self.vbk);
        );
    }
}