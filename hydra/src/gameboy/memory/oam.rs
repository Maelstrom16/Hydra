use std::rc::Rc;

use crate::{common::errors::HydraIOError, gameboy::{Model, ppu::{attributes::TileAttributes, lcdc::ObjectHeight}}};

pub struct Oam {
    inner: [u8; 0x100],
    model: Rc<Model>,
    dma_value: Option<u8>
}

pub const ADDRESS_OFFSET: u16 = 0xFE00;

impl Oam {
    pub fn new(model: Rc<Model>) -> Self {
        Oam { 
            inner: [0; 0x100],
            model,
            dma_value: None
        }
    }

    pub fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        Ok(self.inner[Self::localize_address(address)])
    }

    pub fn get_oam_meta(&self, address: u16) -> ObjectOamMetadata {
        ObjectOamMetadata { address, y: self.inner[Self::localize_address(address)], x: self.inner[Self::localize_address(address + 1)] }
    }

    pub fn resolve_oam_meta(&self, oam_meta: &ObjectOamMetadata) -> ObjectRenderMetadata {
        ObjectRenderMetadata { data_index: self.inner[Self::localize_address(oam_meta.address + 2)], attributes: TileAttributes::from_u8(self.inner[Self::localize_address(oam_meta.address + 3)], &self.model) }
    }

    pub fn write(&mut self, address: u16, value: u8) -> Result<(), HydraIOError> {
        Ok(self.inner[Self::localize_address(address)] = value)
    }

    /// TODO: implement
    fn corruption(&mut self, line: u16) {
        // Corrupt first value in line
        let index = (line - ADDRESS_OFFSET) as usize;
        let a = self.inner[index]; // Current first value
        let b = self.inner[index - 0x8]; // First value from preceding line
        let c = self.inner[index - 0x6]; // Third value from preceding line

        let read_corruption = true; // TODO: replace with actual logic
        if read_corruption {
            // Apply extra corruption if incrementing or decrementing
            let idu_active = true; // TODO: replace with actual logic
            // Only apply corruption if
            if idu_active && (line >= 0xFE20 && line < 0xFE98) {
                let d = self.inner[index - 0x10]; // First value from preceding preceding line
                self.inner[index - 0x8] = (b & (a | c | d)) | (a & c & d);

                // Copy all values from preceding line to its surrounding lines
                for i in (index)..=(index + 0x7) {
                    self.inner[i] = self.inner[i - 0x8];
                    self.inner[i - 0x10] = self.inner[i - 0x8];
                }
            }
            // Standard read corruption
            self.inner[index] = b | (a & c);
        } else {
            self.inner[index] = ((a ^ c) & (b ^ c)) ^ c;
        }

        // Copy last three values from preceding line
        for i in (index + 0x5)..=(index + 0x7) {
            self.inner[i] = self.inner[i - 0x8];
        }
    }

    const fn localize_address(address: u16) -> usize {
        (address - ADDRESS_OFFSET) as usize
    }
}

pub struct ObjectOamMetadata {
    pub address: u16,
    pub y: u8,
    pub x: u8,
}

impl ObjectOamMetadata {
    pub fn occupies_x(&self, x: u8) -> bool {
        ((self.x - 8)..(self.x)).contains(&x)
    }

    pub fn occupies_y(&self, y: u8, obj_height: ObjectHeight) -> bool {
        ((self.y - 16)..(self.y - 16 + obj_height as u8)).contains(&y)
    }
}

pub struct ObjectRenderMetadata {
    pub data_index: u8,
    pub attributes: TileAttributes,
}