use std::rc::Rc;

use crate::{deserialize, gameboy::{Model, memory::{MemoryMappedIo, io::MMIO}}, serialize};

pub struct ColorMap {
    bgp: [u8; 4]
}

impl ColorMap {
    // TODO: allow colors to be configured by user
    const COLOR_MAP: [[u8; 4]; 4] = [
        [255, 255, 255, 255],
        [170, 170, 170, 255],
        [85, 85, 85, 255],
        [0, 0, 0, 255],
    ];

    pub fn new(model: &Rc<Model>) -> Self {
        match model.is_monochrome() { 
            true => ColorMap { bgp: [0b00, 0b11, 0b11, 0b11] },
            false => panic!("GBC Palette mapping not yet supported"),
        }
    }

    pub fn get_color(&self, index: u8) -> &'static [u8] {
        &Self::COLOR_MAP[self.bgp[index as usize] as usize]
    }
}

impl MemoryMappedIo<{MMIO::BGP as u16}> for ColorMap {
    fn read(&self) -> u8 {
        serialize!(
            (self.bgp[3]) =>> 7..=6;
            (self.bgp[2]) =>> 5..=4;
            (self.bgp[1]) =>> 3..=2;
            (self.bgp[0]) =>> 1..=0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7..=6 =>> (self.bgp[3]);
            5..=4 =>> (self.bgp[2]);
            3..=2 =>> (self.bgp[1]);
            1..=0 =>> (self.bgp[0]);
        );
    }
}