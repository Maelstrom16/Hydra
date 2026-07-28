use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use image::imageops::FilterType;
use nokhwa::Camera;
use nokhwa::pixel_format::{LumaFormat, RgbAFormat, RgbFormat};
use nokhwa::utils::Resolution;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, PollType, Queue, ShaderModuleDescriptor, ShaderSource, ShaderStages};

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
const IMAGE_BUFFER_SIZE: usize = SENSOR_BUFFER_SIZE / 4;

const TILE_SIZE: usize = 8;

const DITHER_MATRIX_WIDTH: usize = 4;
const DITHER_MATRIX_HEIGHT: usize = 4;
const DITHER_THRESHOLDS: usize = 3;
const DITHER_MATRIX_SIZE: usize = DITHER_MATRIX_WIDTH * DITHER_MATRIX_HEIGHT * DITHER_THRESHOLDS * 4;

pub struct PocketCamera {
    rom: Rom<0x4000>,
    ram: Sram<0x2000>,

    ram_write_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,

    cam_selected: bool,
    capture_in_progress: Arc<AtomicBool>,
    camera: Camera,
    sensor_buffer: Buffer,
    image_buffer: Buffer,
    gain: f32,
    add_six: bool,
    exposure_time: u16,
    h_enhance: bool,
    v_enhance: bool,
    enhance_ratio: f32,
    invert: bool,
    voltage: f32,
    dithering_thresholds: Buffer,
    rtc_latch: u8,

    device: Arc<Device>,
    queue: Arc<Queue>,
    staging_buffer: Buffer,
    bind_group: BindGroup,
    compute_pipeline: ComputePipeline
}

impl PocketCamera {
    pub fn from_header(header: RomHeader, device: Arc<Device>, queue: Arc<Queue>) -> Result<Self, HydraIOError> {
        let dithering_thresholds = device.create_buffer(&BufferDescriptor {
            label: Some("POCKETCAMERA Dithering Thresholds Matrix"),
            mapped_at_creation: false,
            size: DITHER_MATRIX_SIZE as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let sensor_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("POCKETCAMERA Sensor Output Buffer"),
            mapped_at_creation: false,
            size: SENSOR_BUFFER_SIZE as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        let image_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("POCKETCAMERA Dithered Output Buffer"),
            mapped_at_creation: false,
            size: IMAGE_BUFFER_SIZE as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        let staging_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("POCKETCAMERA Dithered Output Staging Buffer"),
            mapped_at_creation: true,
            size: IMAGE_BUFFER_SIZE as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Uniform, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("POCKETCAMERA Dithering Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(dithering_thresholds.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(sensor_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(image_buffer.as_entire_buffer_binding()),
                },
            ],
            label: Some("POCKETCAMERA Bind Group"),
        });
        
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("POCKETCAMERA Dithering Shader"),
            source: ShaderSource::Wgsl(include_str!("../../../../../shader/gameboy/pocketcamera_dither.wgsl").into()),
        });
        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("POCKETCAMERA Dithering Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("POCKETCAMERA Dithering Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("apply_dither_matrix"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None
        });

        let mut camera = input::initialize_camera()?;
        camera.open_stream()?;

        Ok(PocketCamera {
            ram: Sram::from_header(&header)?,
            rom: header.into_rom(),

            ram_write_enabled: false,
            rom_bank: 1,
            ram_bank: 0,

            cam_selected: false,
            capture_in_progress: Arc::new(false.into()),
            camera,
            image_buffer,
            sensor_buffer,
            gain: 14.0,
            add_six: false,
            exposure_time: 0x0000,
            h_enhance: false,
            v_enhance: false,
            enhance_ratio: 0.50,
            invert: false,
            voltage: 0.0,
            dithering_thresholds,
            rtc_latch: 0xFF,

            device,
            queue,
            staging_buffer,
            bind_group,
            compute_pipeline
        })
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
        } else if !self.capture_in_progress.load(Ordering::Relaxed) {
            match address {
                0xA100..=0xAEFF => {
                    let localized_address = address - 0xA100;
                    let lower_bound = localized_address as u64 & 0xFFF8;
                    let upper_bound = lower_bound + 8;
                    let offset = localized_address as usize & 0b111;

                    Ok(self.staging_buffer.get_mapped_range(lower_bound..upper_bound)[offset])
                }
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
                    if value.test_bit(0) {
                        // Get webcam frame and crop to a square
                        let webcam_view = self.camera.frame().unwrap().decode_image::<LumaFormat>().unwrap();
                        let (webcam_x, webcam_y) = webcam_view.dimensions();
                        let webcam_short = std::cmp::min(webcam_x, webcam_y);
                        let webcam_view_cropped = image::imageops::crop_imm(&webcam_view, (webcam_x - webcam_short) / 2, (webcam_y - webcam_short) / 2, webcam_short, webcam_short).to_image();
                        
                        // Resize to Game Boy Camera dimensions and crop top/bottom rows
                        let sensor_view = image::imageops::resize(&webcam_view_cropped, SENSOR_WIDTH as u32, SENSOR_HEIGHT_UNCROPPED as u32, FilterType::Nearest);
                        let sensor_view_cropped = image::imageops::crop_imm(&sensor_view, 0, TILE_SIZE as u32, SENSOR_WIDTH as u32, SENSOR_HEIGHT as u32).to_image();

                        // Process edge enhancement
                        // let sensor_view_enhanced = image::imageops::filter3x3(&sensor_view_cropped, &[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0]);

                        self.queue.write_buffer(&self.sensor_buffer, 0, &sensor_view_cropped);

                        // Apply dithering thru compute shader
                        let mut command_encoder = self.device.create_command_encoder(&Default::default());

                        let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                            label: None,
                            timestamp_writes: None
                        });
                        compute_pass.set_bind_group(0, &self.bind_group, &[]);
                        compute_pass.set_pipeline(&self.compute_pipeline);
                        compute_pass.dispatch_workgroups(SENSOR_WIDTH_TILES as u32, SENSOR_HEIGHT_TILES as u32, 1);
                        drop(compute_pass);
                        
                        self.staging_buffer.unmap();
                        self.capture_in_progress.store(true, Ordering::Relaxed);
                        command_encoder.copy_buffer_to_buffer(
                            &self.image_buffer,
                            0,
                            &self.staging_buffer,
                            0,
                            Some(IMAGE_BUFFER_SIZE as u64),
                        );

                        self.queue.submit([command_encoder.finish()]);
                        
                        self.device.poll(wgpu::PollType::Wait);
                        let status_clone = self.capture_in_progress.clone();
                        self.staging_buffer.map_async(wgpu::MapMode::Read, .., move |_| {
                            status_clone.store(false, Ordering::Relaxed);
                        });
                    }
                    Ok(())
                }
                0xA001 => {Err(HydraIOError::OpenBusAccess)}
                0xA002 => {Err(HydraIOError::OpenBusAccess)}
                0xA003 => {Err(HydraIOError::OpenBusAccess)}
                0xA004 => {Err(HydraIOError::OpenBusAccess)}
                0xA005 => {Err(HydraIOError::OpenBusAccess)}
                0xA006..=0xA035 => {
                    let localized_address = address as u64 - 0xA006;
                    // The exact address is recalculated to better suit the shader
                    let relocalized_address = (16 * (localized_address % 3)) + (4 * ((localized_address / 12) % 4)) + ((localized_address / 3) % 4);
                    self.queue.write_buffer(&self.dithering_thresholds, relocalized_address * 4, &(value as u32).to_le_bytes());
                    self.queue.submit([]);
                    Ok(())
                }
                _ => {Err(HydraIOError::OpenBusAccess)}
            }
        } else if !self.capture_in_progress.load(Ordering::Relaxed) {
            let BankedAddress { address, bank } = self.localize_ram_address(address);
            Ok(self.ram.write_bank(value, address, bank))
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}