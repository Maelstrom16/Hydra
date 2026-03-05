use std::rc::Rc;

use crate::{deserialize, gameboy::{GbMode, Model}};

pub struct TileAttributes {
    pub(super) bg_priority: bool,
    pub(super) y_flip: bool,
    pub(super) x_flip: bool,
    pub(super) bank_index: u8,
    pub(super) palette: u8,
}

impl TileAttributes {
    pub fn from_u8(val: u8, mode: &Rc<GbMode>) -> Self {
        match **mode {
            GbMode::DMG => {
                deserialize!(val;
                    [7] as bool =>> bg_priority;
                    [6] as bool =>> y_flip;
                    [5] as bool =>> x_flip;
                    [4] =>> palette;
                );
                TileAttributes { bg_priority, y_flip, x_flip, bank_index: 0, palette }
            }
            GbMode::CGB => {
                deserialize!(val;
                    [7] as bool =>> bg_priority;
                    [6] as bool =>> y_flip;
                    [5] as bool =>> x_flip;
                    [3] =>> bank_index;
                    [2..=0] =>> palette;
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