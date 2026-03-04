use std::{cell::RefCell, rc::Rc};

use crate::{common::errors::HydraIOError, deserialize, gameboy::{GbMode, memory::MemoryMapped}, serialize};

pub type Color = [u8; 4];
type DmgPaletteIndices = [u8; 4];

pub const LCD_OFF_COLOR: Color = [255, 255, 255, 255];

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;
const A: usize = 3;

const WHITE: [u8; 4] = [0xF8, 0xF8, 0xF8, 0xFF];
const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];

pub trait ColorMap: MemoryMapped {
    fn get_tile_color(&self, palette_index: u8, color_index: u8) -> Color;
    fn get_object_color(&self, palette_index: u8, color_index: u8) -> Color;
}

pub fn from_mode(mode: &GbMode) -> Box<dyn ColorMap> {
    match mode {
        GbMode::DMG => Box::new(DmgColorMap::new()),
        GbMode::CGB => Box::new(CgbColorMap::new()),
    }
}

pub struct DmgColorMap {
    bg_palette: DmgPaletteIndices,
    ob_palettes: [DmgPaletteIndices; 2],
}


impl DmgColorMap {
    // TODO: allow colors to be configured by user
    const COLOR_MAP: [Color; 4] = [
        [255, 255, 255, 255],
        [170, 170, 170, 255],
        [85, 85, 85, 255],
        [0, 0, 0, 255],
    ];

    fn new() -> Self {
        DmgColorMap {
            bg_palette: [0b00, 0b11, 0b11, 0b11],
            ob_palettes: [[0b11, 0b11, 0b11, 0b11]; 2],
        }
    }

    pub fn read_bgp(&self) -> u8 {
        serialize!(
            (self.bg_palette[3]) =>> [7..=6];
            (self.bg_palette[2]) =>> [5..=4];
            (self.bg_palette[1]) =>> [3..=2];
            (self.bg_palette[0]) =>> [1..=0];
        )
    }

    pub fn write_bgp(&mut self, val: u8) {
        deserialize!(val;
            [7..=6] =>> (self.bg_palette[3]);
            [5..=4] =>> (self.bg_palette[2]);
            [3..=2] =>> (self.bg_palette[1]);
            [1..=0] =>> (self.bg_palette[0]);
        );
    }
    
    pub fn read_obp(&self, palette_index: usize) -> u8 {
        serialize!(
            (self.ob_palettes[palette_index][3]) =>> [7..=6];
            (self.ob_palettes[palette_index][2]) =>> [5..=4];
            (self.ob_palettes[palette_index][1]) =>> [3..=2];
            (self.ob_palettes[palette_index][0]) =>> [1..=0];
        )
    }

    pub fn write_obp(&mut self, val: u8, palette_index: usize) {
        deserialize!(val;
            [7..=6] =>> (self.ob_palettes[palette_index][3]);
            [5..=4] =>> (self.ob_palettes[palette_index][2]);
            [3..=2] =>> (self.ob_palettes[palette_index][1]);
            [1..=0] =>> (self.ob_palettes[palette_index][0]);
        );
    }
}

impl ColorMap for DmgColorMap {
    fn get_tile_color(&self, _palette_index: u8, color_index: u8) -> Color {
        Self::COLOR_MAP[self.bg_palette[color_index as usize] as usize]
    }

    fn get_object_color(&self, palette_index: u8, color_index: u8) -> Color {
        Self::COLOR_MAP[self.ob_palettes[palette_index as usize][color_index as usize] as usize]
    }
}

impl MemoryMapped for DmgColorMap {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF47 => Ok(self.read_bgp()),
            0xFF48 => Ok(self.read_obp(0)),
            0xFF49 => Ok(self.read_obp(1)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF47 => Ok(self.write_bgp(val)),
            0xFF48 => Ok(self.write_obp(val, 0)),
            0xFF49 => Ok(self.write_obp(val, 1)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

pub struct CgbColorMap {
    background: CgbPaletteBank,
    objects: CgbPaletteBank,
}

impl CgbColorMap {
    fn new() -> Self {
        CgbColorMap { 
            background: CgbPaletteBank {
                palettes: [[WHITE; 4]; 8], 
                palette_index: 0b001000, 
                index_auto_increment: true, 
            },
            objects: CgbPaletteBank { 
                palettes: [[BLACK, WHITE, WHITE, WHITE]; 8],
                palette_index: 0b010000, 
                index_auto_increment: true,
            },
        }
    }
}

impl ColorMap for CgbColorMap {
    fn get_tile_color(&self, palette_index: u8, color_index: u8) -> Color {
        self.background.get_color(palette_index, color_index)
    }

    fn get_object_color(&self, palette_index: u8, color_index: u8) -> Color {
        self.objects.get_color(palette_index, color_index)
    }
}

impl MemoryMapped for CgbColorMap {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF68 => Ok(self.background.read_index()),
            0xFF69 => Ok(self.background.read_data()),
            0xFF6A => Ok(self.objects.read_index()),
            0xFF6B => Ok(self.objects.read_data()),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF68 => Ok(self.background.write_index(val)),
            0xFF69 => Ok(self.background.write_data(val)),
            0xFF6A => Ok(self.objects.write_index(val)),
            0xFF6B => Ok(self.objects.write_data(val)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

struct CgbPaletteBank {
    palettes: [[Color; 4]; 8],

    palette_index: u8,
    index_auto_increment: bool,
}

impl CgbPaletteBank {
    fn get_color(&self, palette_index: u8, color_index: u8) -> Color {
        self.palettes[palette_index as usize][color_index as usize]
    }

    pub fn read_index(&self) -> u8 {
        serialize!(
            (self.index_auto_increment as u8) =>> [7];
            0b01000000;
            (self.palette_index) =>> [5..=0];
        )
    }

    pub fn write_index(&mut self, val: u8) {
        deserialize!(val;
            [7] as bool =>> (self.index_auto_increment);
            [5..=0] =>> (self.palette_index);
        );
    }

    pub fn read_data(&self) -> u8 {
        let palette_index = self.palette_index as usize / 8;
        let color_index = (self.palette_index as usize % 8) / 2;
        let color = &self.palettes[palette_index][color_index];

        match self.palette_index % 2 {
            0 => serialize!(
                (color[G] >> 3) =>> [7..=5];
                (color[R] >> 3) =>> [4..=0];
            ),
            1 => serialize!(
                0b10000000;
                (color[B] >> 3) =>> [6..=2];
                (color[G] >> 6) =>> [1..=0];
            ),
            _ => unreachable!()
        }
    }

    pub fn write_data(&mut self, val: u8) {
        let palette_index = self.palette_index as usize / 8;
        let color_index = (self.palette_index as usize % 8) / 2;
        let color = &mut self.palettes[palette_index][color_index];

        match self.palette_index % 2 {
            0 => {
                deserialize!(val;
                    [7..=5] =>> g_lo;
                    [4..=0] =>> r;
                );
                color[G] = (color[G] & 0b11000000) | (g_lo << 3);
                color[R] = r << 3;
            }
            1 => {
                deserialize!(val;
                    [6..=2] =>> b;
                    [1..=0] =>> g_hi;
                );
                color[B] = b << 3;
                color[G] = (color[G] & 0b00111000) | (g_hi << 6);
            }
            _ => unreachable!()
        }

        if self.index_auto_increment {
            self.palette_index = (self.palette_index + 1) % 64;
        }
    }
}