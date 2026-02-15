use crate::{deserialize, gameboy::memory::{MemoryMappedIo, io::MMIO}, serialize};

pub struct LcdController {
    pub(super) lcd_enabled: bool,
    pub(super) window_enabled: bool,
    pub(super) objects_enabled: bool,
    pub(super) tilemaps_enabled: bool,

    pub(super) bg_map_area: TileMapArea,
    pub(super) win_map_area: TileMapArea,
    pub(super) tilemaps_data_area: TileLowDataArea,

    pub(super) object_size: ObjectHeight
}

impl LcdController {
    pub fn new() -> Self {
        LcdController { 
            lcd_enabled: true,
            window_enabled: false,
            objects_enabled: false,
            tilemaps_enabled: true,

            bg_map_area: TileMapArea::Map0,
            win_map_area: TileMapArea::Map0,
            tilemaps_data_area: TileLowDataArea::Data1,

            object_size: ObjectHeight::Standard
        }
    }
}

impl MemoryMappedIo<{MMIO::LCDC as u16}> for LcdController {
    fn read(&self) -> u8 {
        serialize!(
            (self.lcd_enabled as u8) =>> 7;
            ((self.win_map_area == TileMapArea::Map1) as u8) =>> 6;
            (self.window_enabled as u8) =>> 5;
            ((self.tilemaps_data_area == TileLowDataArea::Data1) as u8) =>> 4;
            ((self.bg_map_area == TileMapArea::Map1) as u8) =>> 3;
            ((self.object_size == ObjectHeight::Tall) as u8) =>> 2;
            (self.objects_enabled as u8) =>> 1;
            (self.tilemaps_enabled as u8) =>> 0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> (self.lcd_enabled);
            6 as bool =>> win_map_area;
            5 as bool =>> (self.window_enabled);
            4 as bool =>> tilemaps_data_area;
            3 as bool =>> bg_map_area;
            2 as bool =>> object_size;
            1 as bool =>> (self.objects_enabled);
            0 as bool =>> (self.tilemaps_enabled);
        );
        self.win_map_area = TileMapArea::from_bool(win_map_area);
        self.tilemaps_data_area = TileLowDataArea::from_bool(tilemaps_data_area);
        self.bg_map_area = TileMapArea::from_bool(bg_map_area);
        self.object_size = ObjectHeight::from_bool(object_size);
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum TileMapArea {
    Map0 = 0x9800,
    Map1 = 0x9C00,
}
impl TileMapArea {
    pub fn from_bool(cond: bool) -> Self {
        if cond {TileMapArea::Map1} else {TileMapArea::Map0}
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum TileLowDataArea {
    Data0 = 0x9000,
    Data1 = 0x8000,
}
impl TileLowDataArea {
    pub fn from_bool(cond: bool) -> Self {
        if cond {TileLowDataArea::Data1} else {TileLowDataArea::Data0}
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ObjectHeight {
    Standard = 8,
    Tall = 16
}
impl ObjectHeight {
    pub fn from_bool(cond: bool) -> Self {
        if cond {ObjectHeight::Tall} else {ObjectHeight::Standard}
    }
}