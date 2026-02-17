use crate::{deserialize, gameboy::ppu::lcdc::ObjectHeight};

pub struct TileAttributes {
    pub(super) priority: bool,
    pub(super) y_flip: bool,
    pub(super) x_flip: bool,
    pub(super) bank_index: u8,
    pub(super) palette: u8,
}

impl TileAttributes {
    pub fn from_u8(val: u8) -> Self {
        deserialize!(val;
            7 as bool =>> priority;
            6 as bool =>> y_flip;
            5 as bool =>> x_flip;
            3 =>> bank_index;
            2..=0 =>> palette;
        );

        TileAttributes { priority, y_flip, x_flip, bank_index, palette }
    }
}

pub struct ObjectAttributes {
    pub(super) y: u8,
    pub(super) x: u8,
    pub(super) data_index: u8,
    pub(super) attributes: TileAttributes,
}

impl ObjectAttributes {
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        ObjectAttributes {
            y: bytes[0],
            x: bytes[1],
            data_index: bytes[2],
            attributes: TileAttributes::from_u8(bytes[3]),
        }
    }

    pub fn occupies_scanline(&self, scanline: u8, obj_height: ObjectHeight) -> bool {
        let upper = scanline + 16;
        ((upper - obj_height as u8 + 1)..=upper).contains(&self.y)
    }
}