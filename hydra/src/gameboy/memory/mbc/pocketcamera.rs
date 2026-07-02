use std::time::{Duration, Instant};

use image::imageops::FilterType;
use nokhwa::Camera;
use nokhwa::pixel_format::{LumaFormat, RgbAFormat, RgbFormat};
use nokhwa::utils::Resolution;

use crate::common::bit::BitVec;
use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};
use crate::{deserialize, input, serialize};

const SENSOR_WIDTH_TILES: usize = 16;
const SENSOR_WIDTH: usize = SENSOR_WIDTH_TILES * TILE_SIZE;
const SENSOR_HEIGHT_TILES: usize = 14;
const SENSOR_HEIGHT_TILES_UNCROPPED: usize = 16;
const SENSOR_HEIGHT: usize = SENSOR_HEIGHT_TILES * TILE_SIZE;
const SENSOR_HEIGHT_UNCROPPED: usize = SENSOR_HEIGHT_TILES_UNCROPPED * TILE_SIZE;
const SENSOR_BUFFER_SIZE: usize = SENSOR_WIDTH * SENSOR_HEIGHT;

const TILE_SIZE: usize = 8;

pub struct PocketCamera {
    rom: Rom<0x4000>,
    ram: Sram<0x2000>,

    ram_write_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,

    cam_selected: bool,
    capture_in_progress: bool,
    camera: Camera,
    image_buffer: [u8; SENSOR_BUFFER_SIZE],
    gain: f32,
    add_six: bool,
    exposure_time: u16,
    h_enhance: bool,
    v_enhance: bool,
    enhance_ratio: f32,
    invert: bool,
    voltage: f32,
    rtc_latch: u8,
}

impl PocketCamera {
    pub fn from_header(header: RomHeader) -> Result<Self, HydraIOError> {
        let mut result = PocketCamera {
            ram: Sram::from_header(&header)?,
            rom: header.into_rom(),

            ram_write_enabled: false,
            rom_bank: 1,
            ram_bank: 0,

            cam_selected: false,
            capture_in_progress: false,
            camera: input::initialize_camera()?,
            image_buffer: [0x00; SENSOR_BUFFER_SIZE],
            gain: 14.0,
            add_six: false,
            exposure_time: 0x0000,
            h_enhance: false,
            v_enhance: false,
            enhance_ratio: 0.50,
            invert: false,
            voltage: 0.0,
            rtc_latch: 0xFF,
        };

        println!("{:?}", result.camera.compatible_camera_formats());
        let e = result.camera.set_frame_format(nokhwa::utils::FrameFormat::GRAY);
        println!("{:?}", e);
        result.camera.open_stream()?;

        Ok(result)
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => panic!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn localize_ram_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0xA000..=0xBFFF => BankedAddress {address: address - sram::ADDRESS_OFFSET as u16, bank: self.ram_bank as usize % self.ram.get_bank_count()},
            _ => panic!("Attempted to localize invalid RAM address {}", address)
        }
    }
}

impl mbc::MemoryBankController for PocketCamera {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.cam_selected {
            Ok(0x00)
            // Err(HydraIOError::OpenBusAccess)
        } else if !self.capture_in_progress {
            match address {
                0xA100..=0xAEFF => Ok(self.image_buffer[address as usize - 0xA100]),
                _ => {
                    let BankedAddress { address, bank } = self.localize_ram_address(address);
                    Ok(self.ram.read_bank(address, bank))
                }
            }
        } else {
            Ok(0x00)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address {
            0x0000..=0x1FFF => match value {
                0x00 => self.ram_write_enabled = false,
                0x0A => self.ram_write_enabled = true,
                _ => { /* Leave RAM in current state */ }
            }
            0x2000..=0x3FFF => {self.rom_bank = value & 0b111111}
            0x4000..=0x5FFF => {
                deserialize!(value;
                    [4] as bool =>> (self.cam_selected);
                    [3..=0] =>> (self.ram_bank);
                );
            }
            0x6000..=0x7FFF => { /* Do nothing */ }
            _ => panic!("Invalid ROM address")
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.cam_selected {
            match address {
                0xA000 => {
                    // Get webcam frame and crop to a square
                    let webcam_view = self.camera.frame().unwrap().decode_image::<LumaFormat>().unwrap();
                    let (webcam_x, webcam_y) = webcam_view.dimensions();
                    let webcam_short = std::cmp::min(webcam_x, webcam_y);
                    let webcam_view_cropped = image::imageops::crop_imm(&webcam_view, (webcam_x - webcam_short) / 2, (webcam_y - webcam_short) / 2, webcam_short, webcam_short).to_image();
                    // Resize to Game Boy Camera dimensions and crop top/bottom rows
                    let sensor_view = image::imageops::resize(&webcam_view_cropped, SENSOR_WIDTH as u32, SENSOR_HEIGHT_UNCROPPED as u32, FilterType::Nearest);
                    let sensor_view_cropped = image::imageops::crop_imm(&sensor_view, 0, TILE_SIZE as u32, SENSOR_WIDTH as u32, SENSOR_HEIGHT as u32).to_image();
                    // Convert to Game Boy tile data
                    for (i, pixels) in sensor_view_cropped.as_chunks::<8>().0.into_iter().enumerate() {
                        let bitplanes = pixels.iter().fold((0, 0), |(b0, b1), pixel| {
                            ((b0 << 1) | !pixel.test_bit(6) as u8, (b1 << 1) | !pixel.test_bit(7) as u8)
                        });
                        let tile_x = i % SENSOR_WIDTH_TILES;
                        let tile_y = i / (SENSOR_WIDTH_TILES * TILE_SIZE);
                        let y = (i / SENSOR_WIDTH_TILES) % TILE_SIZE;
                        let buffer_index = ((tile_y * SENSOR_WIDTH_TILES * TILE_SIZE) + (tile_x * TILE_SIZE) + y) * 2;
                        self.image_buffer[buffer_index] = bitplanes.0;
                        self.image_buffer[buffer_index + 1] = bitplanes.1;
                    }
                    Ok(())
                }
                0xA001 => {Err(HydraIOError::OpenBusAccess)}
                0xA002 => {Err(HydraIOError::OpenBusAccess)}
                0xA003 => {Err(HydraIOError::OpenBusAccess)}
                0xA004 => {Err(HydraIOError::OpenBusAccess)}
                0xA005 => {Err(HydraIOError::OpenBusAccess)}
                0xA006..=0xA035 => {Err(HydraIOError::OpenBusAccess)}
                _ => {Err(HydraIOError::OpenBusAccess)}
            }
        } else if !self.capture_in_progress {
            let BankedAddress { address, bank } = self.localize_ram_address(address);
            Ok(self.ram.write_bank(value, address, bank))
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}