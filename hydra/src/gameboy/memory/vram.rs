mod deserialized;

use std::cell::RefCell;
use std::rc::Rc;

use crate::common::errors::HydraIOError;
use crate::gameboy::Model;
use crate::gameboy::ppu::PpuMode;

pub const ADDRESS_OFFSET: u16 = 0x8000;

pub struct Vram {
    model: Rc<Model>,
    vram: Box<[[u8; 0x2000]]>,
    vbk: u8,
    ppu_mode: Rc<RefCell<PpuMode>>,
}

impl Vram {
    pub fn new(model: Rc<Model>, ppu_mode: Rc<RefCell<PpuMode>>) -> Self {
        let bank_count = match model.is_monochrome() {
            true => 1,
            false => 2
        };

        Vram {
            model,
            vram: vec![[0; 0x2000]; bank_count].into_boxed_slice(),
            vbk: 0,
            ppu_mode,
        }
    }

    // TODO: Reenable VRAM inaccessbility when timing is fixed
    pub fn read_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        // if self.is_accessible() {
            Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)])
        // } else {
        //     Err(HydraIOError::OpenBusAccess)
        // }
    }

    pub fn write_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        // if self.is_accessible() {
            Ok(self.vram[self.get_bank_id() as usize][Vram::localize_address(address)] = value)
        // } else {
        //     Err(HydraIOError::OpenBusAccess)
        // }
    }

    pub fn unbound_read_u8(&self, address: u16, bank: u8) -> u8 {
        self.vram[bank as usize][Vram::localize_address(address)]
    }

    fn is_accessible(&self) -> bool {
        // VRAM is inaccessible during PPU mode 3
        *self.ppu_mode.borrow() != PpuMode::Render
    }

    fn get_bank_id(&self) -> u8 {
        if self.model.is_monochrome() {0} else {self.vbk}
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}