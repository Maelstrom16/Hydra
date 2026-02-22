use std::cell::RefCell;
use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::gameboy::Model;
use crate::gameboy::memory::{MMIO, MemoryMappedIo};
use crate::gameboy::ppu::PpuMode;
use crate::gameboy::ppu::attributes::TileAttributes;
use crate::gameboy::ppu::lcdc::LcdController;
use crate::{deserialize, serialize};

pub const ADDRESS_OFFSET: u16 = 0x8000;

pub struct Vram {
    model: Rc<Model>,
    vram: Box<[[u8; 0x2000]]>,
    vbk: u8,
    ppu_mode: Rc<RefCell<PpuMode>>,
    lcdc: Rc<RefCell<LcdController>>
}

impl Vram {
    pub fn new(model: Rc<Model>, ppu_mode: Rc<RefCell<PpuMode>>, lcdc: Rc<RefCell<LcdController>>) -> Self {
        let bank_count = match model.is_monochrome() {
            true => 1,
            false => 2
        };

        Vram {
            model,
            vram: vec![[0; 0x2000]; bank_count].into_boxed_slice(),
            vbk: 0,
            ppu_mode,
            lcdc
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

    pub fn read_tile_data(&self, address: u16, bank: u8) -> u8 {
        self.vram[bank as usize][Vram::localize_address(address)]
    }

    pub fn read_tile_map(&self, address: u16) -> (u8, TileAttributes) {
        let address = Vram::localize_address(address);
        (self.vram[0][address], match self.model.is_monochrome() {
            true => TileAttributes::default(),
            false => TileAttributes::from_u8(self.vram[1][address], &self.model)
        }) 
    }

    fn is_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3 when LCD is enabled
        *self.ppu_mode.borrow() != PpuMode::Render || !self.lcdc.borrow().is_lcd_enabled()
    }

    fn get_bank_id(&self) -> u8 {
        if self.model.is_monochrome() {0} else {self.vbk}
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}

impl MemoryMappedIo<{MMIO::VBK as u16}> for Vram {
    fn read(&self) -> u8 {
        serialize!(
            0b11111110;
            (self.vbk) =>> 0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            0 =>> (self.vbk);
        );
    }
}