use crate::deserialize;

pub struct TileAttributes {
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    bank_index: u8,
    palette: u8,
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