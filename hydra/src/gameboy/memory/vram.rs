use std::cell::RefCell;
use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::gameboy::{GbMode, Model};
use crate::gameboy::ppu::PpuMode;
use crate::gameboy::ppu::attributes::TileAttributes;
use crate::{deserialize, serialize};

pub const ADDRESS_OFFSET: u16 = 0x8000;

pub struct Vram {
    model: Rc<Model>,
    mode: Rc<GbMode>,
    vram: Box<[[u8; 0x2000]]>,
    vbk: u8,
}

impl Vram {
    pub fn new(model: Rc<Model>, mode: Rc<GbMode>) -> Self {
        let bank_count = match *mode {
            GbMode::DMG => 1,
            GbMode::CGB => 2
        };

        Vram {
            model,
            mode,
            vram: vec![[0; 0x2000]; bank_count].into_boxed_slice(),
            vbk: 0,
        }
    }

    pub fn read_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)])
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)] = value)
    }

    pub fn read_tile_data(&self, address: u16, bank: u8) -> u8 {
        self.vram[bank as usize][Vram::localize_address(address)]
    }

    pub fn read_tile_map(&self, address: u16) -> (u8, TileAttributes) {
        let address = Vram::localize_address(address);
        (self.vram[0][address], match *self.mode {
            GbMode::DMG => TileAttributes::default(),
            GbMode::CGB => TileAttributes::from_u8(self.vram[1][address], &self.mode)
        }) 
    }

    fn get_bank_id(&self) -> u8 {
        if matches!(*self.mode, GbMode::DMG) {0} else {self.vbk}
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}

impl Vram {
    pub fn read_vbk(&self) -> Result<u8, HydraIOError> {
        // Disallow reads on DMG hardware
        if self.model.is_monochrome() {return Err(HydraIOError::OpenBusAccess)}
        Ok(serialize!(
            0b11111110;
            (self.vbk) =>> [0];
        ))
    }

    pub fn write_vbk(&mut self, val: u8) -> Result<(), HydraIOError> {
        // Disallow writes in DMG mode
        if matches!(*self.mode, GbMode::DMG) {return Err(HydraIOError::OpenBusAccess)}
        deserialize!(val;
            [0] =>> (self.vbk);
        );
        Ok(())
    }
}