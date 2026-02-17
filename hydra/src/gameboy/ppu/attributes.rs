use std::{cell::RefCell, rc::Rc};

use crate::{deserialize, gameboy::{Model, memory::oam::Oam, ppu::lcdc::ObjectHeight}};

pub struct TileAttributes {
    pub(super) bg_priority: bool,
    pub(super) y_flip: bool,
    pub(super) x_flip: bool,
    pub(super) bank_index: u8,
    pub(super) palette: u8,
}

impl TileAttributes {
    pub fn from_u8(val: u8, model: &Rc<Model>) -> Self {
        match model.is_monochrome() {
            true => {
                deserialize!(val;
                    7 as bool =>> bg_priority;
                    6 as bool =>> y_flip;
                    5 as bool =>> x_flip;
                    4 =>> palette;
                );
                TileAttributes { bg_priority, y_flip, x_flip, bank_index: 0, palette }
            }
            false => {
                deserialize!(val;
                    7 as bool =>> bg_priority;
                    6 as bool =>> y_flip;
                    5 as bool =>> x_flip;
                    3 =>> bank_index;
                    2..=0 =>> palette;
                );
                TileAttributes { bg_priority, y_flip, x_flip, bank_index, palette }
            }
        }
    }
}

impl Default for TileAttributes {
    fn default() -> Self {
        TileAttributes { bg_priority: false, y_flip: false, x_flip: false, bank_index: 0, palette: 0 }
    }
}