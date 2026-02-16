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

pub struct ObjectAttributes {
    y: u8,
    x: u8,
    data_index: u8,
    attributes: TileAttributes,
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
}