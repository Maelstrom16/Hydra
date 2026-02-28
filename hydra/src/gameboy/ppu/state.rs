use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{common::{bit::BitVec, errors::HydraIOError}, deserialize, gameboy::{GBRevision, Model, interrupt::{Interrupt, InterruptFlags}, memory::{MemoryMap, MemoryMapped}, ppu::PpuMode}, serialize};

pub struct PpuState {
    pub(super) ppu_mode: PpuMode,
    
    dots: u32,
    ly: u8,
    lyc: u8,

    pub(super) lcd_enabled: bool,
    pub(super) window_enabled: bool,
    pub(super) objects_enabled: bool,
    pub(super) tilemaps_enabled: bool,

    pub(super) bg_map_area: TileMapArea,
    pub(super) win_map_area: TileMapArea,
    pub(super) tilemaps_data_area: TileLowDataArea,

    pub(super) object_size: ObjectHeight,

    pub(super) scy: u8,
    pub(super) scx: u8,
    pub(super) wy: u8,
    pub(super) wx: u8,

    stat_interrupt_select: u8,
}

impl PpuState {
    pub fn new(model: &Rc<Model>) -> Self {
        // Start at beginning of OAM scan for selected ly
        let ly = match **model {
            Model::GameBoy(GBRevision::DMG0) => 0x91,
            Model::GameBoy(_) => 0x00,
            Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles
        };
        PpuState { 
            ppu_mode: PpuMode::default_oam(),

            dots: ly as u32 * Self::DOTS_PER_SCANLINE,
            ly,
            lyc: 0,

            lcd_enabled: true,
            window_enabled: false,
            objects_enabled: false,
            tilemaps_enabled: true,

            bg_map_area: TileMapArea::Map0,
            win_map_area: TileMapArea::Map0,
            tilemaps_data_area: TileLowDataArea::Data1,

            object_size: ObjectHeight::Standard,

            scy: 0x00,
            scx: 0x00,
            wy: 0x00,
            wx: 0x00,

            stat_interrupt_select: 0,
        }
    }

    const DOTS_PER_SCANLINE: u32 = 456;
    const DOTS_PER_FRAME: u32 = 70224;

    pub fn tick(&mut self, interrupt_flags: &mut InterruptFlags) {
        self.dots = (self.dots + 1) % Self::DOTS_PER_FRAME;
        let ly = (self.dots / Self::DOTS_PER_SCANLINE) as u8;
        self.ly_eq_lyc_check(ly == self.lyc, interrupt_flags);
        self.ly = ly;
    }

    fn ly_eq_lyc_check(&mut self, new_ly_eq_lyc: bool, interrupt_flags: &mut InterruptFlags) {
        // Detect rising edge on stat interrupt line
        if (new_ly_eq_lyc && self.ly != self.lyc && self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.as_stat_line_flag())) {
            interrupt_flags.request(Interrupt::Stat);
        }
    }

    pub fn is_lcd_enabled(&self) -> bool {
        self.lcd_enabled
    }

    pub fn get_mode(&self) -> &PpuMode {
        &self.ppu_mode
    }

    pub fn set_mode(&mut self, ppu_mode: PpuMode, interrupt_flags: &mut InterruptFlags) {
        if matches!(ppu_mode, PpuMode::VBlank) {
            interrupt_flags.request(Interrupt::Vblank);
        }

        // Detect rising edge on stat interrupt line
        if (self.ly != self.lyc || !self.stat_interrupt_select.test_bit(6))
        && (!self.stat_interrupt_select.test_bits(self.ppu_mode.as_stat_line_flag()))
        && (self.stat_interrupt_select.test_bits(ppu_mode.as_stat_line_flag())) {
            interrupt_flags.request(Interrupt::Stat);
        }
        
        self.ppu_mode = ppu_mode;
    }

    pub fn get_dots(&self) -> u32 {
        self.dots
    }

    pub fn get_dot_coords(&self) -> (u32, u8) {
        (self.dots % Self::DOTS_PER_SCANLINE, self.ly)
    }
}

impl PpuState {
    
    pub fn read_lcdc(&self) -> u8 {
        serialize!(
            (self.lcd_enabled as u8) =>> 7;
            (matches!(self.win_map_area, TileMapArea::Map1) as u8) =>> 6;
            (self.window_enabled as u8) =>> 5;
            (matches!(self.tilemaps_data_area, TileLowDataArea::Data1) as u8) =>> 4;
            (matches!(self.bg_map_area, TileMapArea::Map1) as u8) =>> 3;
            (matches!(self.object_size, ObjectHeight::Tall) as u8) =>> 2;
            (self.objects_enabled as u8) =>> 1;
            (self.tilemaps_enabled as u8) =>> 0;
        )
    }

    pub fn write_lcdc(&mut self, val: u8) {
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

    pub fn read_stat(&self) -> u8 {
        serialize!(
            0b10000000;
            (self.stat_interrupt_select) => 6..=3;
            ((self.ly == self.lyc) as u8) =>> 2;
            (self.ppu_mode.as_u2()) =>> 1..=0;
        )
    }
    
    pub fn write_stat(&mut self, val: u8) {
        deserialize!(val;
            6..=3 => (self.stat_interrupt_select);
        );
    }
    
    pub fn read_ly(&self) -> u8 {
        self.ly
    }
    
    pub fn write_ly(&mut self, _val: u8) {
        // Do nothing -- readonly
    }
    
    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }
    
    pub fn write_lyc(&mut self, lyc: u8, interrupt_flags: &mut InterruptFlags) {
        self.ly_eq_lyc_check(self.ly == lyc, interrupt_flags);
        self.lyc = lyc;
    }
}

impl PpuState {
    pub fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF40 => Ok(self.read_lcdc()),
            0xFF41 => Ok(self.read_stat()),
            0xFF42 => Ok(self.scy),
            0xFF43 => Ok(self.scx),
            0xFF44 => Ok(self.read_ly()),
            0xFF45 => Ok(self.read_lyc()),
            0xFF4A => Ok(self.wy),
            0xFF4B => Ok(self.wx),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    pub fn write(&mut self, val: u8, address: u16, interrupt_flags: &mut InterruptFlags) -> Result<(), HydraIOError> {
        match address {
            0xFF40 => Ok(self.write_lcdc(val)),
            0xFF41 => Ok(self.write_stat(val)),
            0xFF42 => Ok(self.scy = val),
            0xFF43 => Ok(self.scx = val),
            0xFF44 => Ok(self.write_ly(val)),
            0xFF45 => Ok(self.write_lyc(val, interrupt_flags)),
            0xFF4A => Ok(self.wy = val),
            0xFF4B => Ok(self.wx = val),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub enum ObjectHeight {
    Standard = 8,
    Tall = 16
}
impl ObjectHeight {
    pub fn from_bool(cond: bool) -> Self {
        if cond {ObjectHeight::Tall} else {ObjectHeight::Standard}
    }
}