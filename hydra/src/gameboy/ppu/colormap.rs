use std::rc::Rc;

use crate::{deserialize, gameboy::Model, serialize};

pub struct ColorMap {
    bgp: [u8; 4],
    obp: [[u8; 4]; 2]
}

impl ColorMap {
    // TODO: allow colors to be configured by user
    pub const LCD_OFF_COLOR: [u8; 4] = [0, 0, 0, 255];
    const COLOR_MAP: [[u8; 4]; 4] = [
        [255, 255, 255, 255],
        [170, 170, 170, 255],
        [85, 85, 85, 255],
        [0, 0, 0, 255],
    ];

    pub fn new(model: &Rc<Model>) -> Self {
        match model.is_monochrome() { 
            true => ColorMap { 
                bgp: [0b00, 0b11, 0b11, 0b11],
                obp: [[0b11, 0b11, 0b11, 0b11], [0b11, 0b11, 0b11, 0b11]] // Uninitialized, but 0xFF is a common value
            },
            false => ColorMap { 
                bgp: [0b11, 0b11, 0b11, 0b11],
                obp: [[0b11, 0b11, 0b11, 0b11], [0b11, 0b11, 0b11, 0b11]]
            },
        }
    }

    pub fn get_tile_color(&self, index: u8) -> &'static [u8] {
        &Self::COLOR_MAP[self.bgp[index as usize] as usize]
    }

    pub fn get_object_color(&self, palette_index: u8, color_index: u8) -> &'static [u8] {
        &Self::COLOR_MAP[self.obp[palette_index as usize][color_index as usize] as usize]
    }
}

impl ColorMap {
    pub fn read_bgp(&self) -> u8 {
        serialize!(
            (self.bgp[3]) =>> 7..=6;
            (self.bgp[2]) =>> 5..=4;
            (self.bgp[1]) =>> 3..=2;
            (self.bgp[0]) =>> 1..=0;
        )
    }

    pub fn write_bgp(&mut self, val: u8) {
        deserialize!(val;
            7..=6 =>> (self.bgp[3]);
            5..=4 =>> (self.bgp[2]);
            3..=2 =>> (self.bgp[1]);
            1..=0 =>> (self.bgp[0]);
        );
    }
    
    pub fn read_obp(&self, palette_index: usize) -> u8 {
        serialize!(
            (self.obp[palette_index][3]) =>> 7..=6;
            (self.obp[palette_index][2]) =>> 5..=4;
            (self.obp[palette_index][1]) =>> 3..=2;
            (self.obp[palette_index][0]) =>> 1..=0;
        )
    }

    pub fn write_obp(&mut self, val: u8, palette_index: usize) {
        deserialize!(val;
            7..=6 =>> (self.obp[palette_index][3]);
            5..=4 =>> (self.obp[palette_index][2]);
            3..=2 =>> (self.obp[palette_index][1]);
            1..=0 =>> (self.obp[palette_index][0]);
        );
    }
}