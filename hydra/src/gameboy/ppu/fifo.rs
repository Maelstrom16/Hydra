use crate::gameboy::{memory::{MemoryMap, oam::ObjectOamMetadata}, ppu::{self, SCREEN_WIDTH, attributes::TileAttributes, colormap::{self, Color}, state::{ObjectHeight, PpuState}}};

pub struct FifoFetcher {
    // bg_fifo: [Color; 16],
    // ob_fifo: [Color; 16],

    pub(super) scanline_objects: Vec<ObjectOamMetadata>,

    pub(super) screen_x: u8,
    pub(super) screen_y: u8,
}

impl FifoFetcher {
    pub fn new() -> Self {
        FifoFetcher { 
            // bg_fifo: [], 
            // ob_fifo: (), 

            scanline_objects: Vec::with_capacity(10), 

            screen_x: 0, 
            screen_y: 0 
        }
    }
    
    pub fn resolve_color(&mut self, memory: &mut MemoryMap) -> Color {
        self.screen_y = memory.ppu_state.read_ly();
        let color = self.resolve_color_inner(memory);
        self.screen_x = (self.screen_x + 1) % SCREEN_WIDTH;

        color
    }

    fn resolve_color_inner(&mut self, memory: &mut MemoryMap) -> Color {
        if !memory.ppu_state.lcd_enabled {
            return colormap::LCD_OFF_COLOR;
        }

        // Check BG/window color first
        let (bg_palette_index, bg_color_index, bg_priority) = if memory.ppu_state.tilemaps_enabled { // TODO: use tilemaps_enabled for priority in CGB mode
            let is_window = memory.ppu_state.window_enabled && self.screen_x >= memory.ppu_state.wx.saturating_sub(7) && self.screen_y >= memory.ppu_state.wy;

            let data_low_address = memory.ppu_state.tilemaps_data_area as u16;
            let (map_address, map_x, map_y) = match is_window {
                false => (
                    memory.ppu_state.bg_map_area as u16,
                    self.screen_x.wrapping_add(memory.ppu_state.scx),
                    self.screen_y.wrapping_add(memory.ppu_state.scy),
                ),
                true => (
                    memory.ppu_state.win_map_area as u16,
                    self.screen_x - memory.ppu_state.wx + 7,
                    self.screen_y - memory.ppu_state.wy,
                ),
            };

            let map_index_x = (map_x / 8) as u16;
            let map_index_y = (map_y / 8) as u16;
            let data_index_address = map_address + map_index_x + (map_index_y * ppu::MAP_WIDTH as u16);

            let (data_index, tile_attributes) = memory.vram.read_tile_map(data_index_address);
            let data_address = if data_index < 0x80 {
                data_low_address + (data_index as u16 * 16)
            } else {
                0x8800 + ((data_index - 0x80) as u16 * 16)
            };

            let tile_y = map_y % 8;
            let tile_x = map_x % 8;
            
            (tile_attributes.palette, self.resolve_color_index(tile_x, tile_y, data_address, &tile_attributes, false, memory), tile_attributes.bg_priority)
        } else {
            (0, 0, false)
        };

        // Return early if BG color has priority over any potential objects
        let bg_color = memory.color_map.get_tile_color(bg_palette_index, bg_color_index);
        let bg_can_override = bg_color_index != 0;
        if bg_priority && bg_can_override {
            return bg_color;
        }

        // If background does not have inherent priority, check for opaque object pixels
        let valid_objects = self.scanline_objects.iter().filter(|obj| obj.occupies_x(self.screen_x));
        if memory.ppu_state.objects_enabled { 
            for oam_meta in valid_objects {
                let mut render_meta = memory.oam.resolve_oam_meta(oam_meta);
                // Ignore LSB of tile index if objects are tall
                if matches!(memory.ppu_state.object_size, ObjectHeight::Tall) {
                    render_meta.data_index &= 0b11111110; 
                }

                let data_address = 0x8000 + (render_meta.data_index as u16 * 16);

                let tile_y = self.screen_y + 16 - oam_meta.y;
                let tile_x = self.screen_x - oam_meta.x;
                let obj_color_index = self.resolve_color_index(tile_x, tile_y, data_address, &render_meta.attributes, true, memory);

                if obj_color_index != 0 {
                    return match render_meta.attributes.bg_priority && bg_can_override {
                        true => bg_color,
                        false => memory.color_map.get_object_color(render_meta.attributes.palette, obj_color_index)
                    }
                }
            }
        }

        // Return background color if no opaque object pixels are found
        return bg_color
    }

    fn resolve_color_index(&self, tile_x: u8, tile_y: u8, tile_address: u16, attributes: &TileAttributes, is_object: bool, memory: &mut MemoryMap) -> u8 {
        let tile_x = match attributes.x_flip {
            true => 7 - tile_x,
            false => tile_x
        };
        let tile_y = match attributes.y_flip {
            true => (if !is_object || matches!(memory.ppu_state.object_size, ObjectHeight::Standard) {7} else {15}) - tile_y,
            false => tile_y
        };

        let byte_address = tile_address + (tile_y as u16 * 2);
        let data = [memory.vram.read_tile_data(byte_address, attributes.bank_index),
            memory.vram.read_tile_data(byte_address + 1, attributes.bank_index)];

        let color_index_bits = data.map(|byte| (byte >> (7 - tile_x)) & 1);
        color_index_bits[1] << 1 | color_index_bits[0]
    }
}