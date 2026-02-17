use std::sync::Arc;

use cpal::{Stream, traits::{DeviceTrait, HostTrait}};

use crate::{audio::Audio, common::audio, deserialize, gameboy::memory::{MMIO, MemoryMappedIo}, serialize};

pub struct Pulse {
    enabled: bool,
    stream: Stream,
    
    period: u16,
    period_sweep_interval: u8,
    period_sweep_direction: Direction,
    period_sweep_step: u8,

    volume: u16,
    volume_sweep_interval: u8,
    volume_sweep_direction: Direction,

    duty_cycle: f32,
    length_timer: u8,
    length_timer_enabled: bool,
}

impl Pulse {
    const DUTY_CYCLES: [f32; 4] = [0.125, 0.25, 0.5, 0.75];

    pub fn new1(audio: &Arc<Audio>) -> Self {
        let stream = audio.build_output_stream(audio::sawtooth_callback(440.0, audio), audio::error_callback, None);
        Pulse { 
            enabled: true, 
            stream,

            period: 0b11111111111, 
            period_sweep_interval: 0, 
            period_sweep_direction: Direction::Decreasing, 
            period_sweep_step: 0, 

            volume: 0b1111, 
            volume_sweep_interval: 3, 
            volume_sweep_direction: Direction::Decreasing, 
            
            duty_cycle: 0.5, 
            length_timer: 0b111111, 
            length_timer_enabled: false
        }
    }

    pub fn new2(audio: &Arc<Audio>) -> Self {
        let stream = audio.build_output_stream(audio::sawtooth_callback(261.63, audio), audio::error_callback, None);
        Pulse { 
            enabled: true, 
            stream,

            period: 0b11111111111, 
            period_sweep_interval: 0, 
            period_sweep_direction: Direction::Decreasing, 
            period_sweep_step: 0, 

            volume: 0, 
            volume_sweep_interval: 0, 
            volume_sweep_direction: Direction::Decreasing, 
            
            duty_cycle: 0.125, 
            length_timer: 0b111111, 
            length_timer_enabled: false 
        }
    }
}

impl MemoryMappedIo<{MMIO::NR11 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            (0) =>> 7..=6;
            0b00111111;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7..=6 =>> duty_cycle;
            5..=0 =>> (self.length_timer);
        );
        self.duty_cycle = Self::DUTY_CYCLES[duty_cycle as usize];
    }
}

enum Direction {
    Increasing = 1,
    Decreasing = -1
}

//         MMIO::NR10 => GBReg::new(0x80, 0b01111111, 0b01111111),
//         MMIO::NR11 => GBReg::new(0xBF, 0b11000000, 0b11111111),
//         MMIO::NR12 => GBReg::new(0xF3, 0b11111111, 0b11111111),
//         MMIO::NR13 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR14 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR21 => GBReg::new(0x3F, 0b11000000, 0b11111111),
//         MMIO::NR22 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR23 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR24 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR30 => GBReg::new(0x7F, 0b10000000, 0b10000000),
//         MMIO::NR31 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR32 => GBReg::new(0x9F, 0b01100000, 0b01100000),
//         MMIO::NR33 => GBReg::new(0xFF, 0b11111111, 0b11111111),
//         MMIO::NR34 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR41 => GBReg::new(0xFF, 0b00000000, 0b00111111),
//         MMIO::NR42 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR43 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR44 => GBReg::new(0xBF, 0b01000000, 0b11000000),
//         MMIO::NR50 => GBReg::new(0x77, 0b11111111, 0b11111111),
//         MMIO::NR51 => GBReg::new(0xF3, 0b11111111, 0b11111111),
//         MMIO::NR52 => GBReg::new(match model {
//             Model::GameBoy(_) => 0xF1,
//             Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0xF0,
//         }, 0b10001111, 0b00001111),
//         MMIO::WAV00 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV01 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV02 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV03 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV04 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV05 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV06 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV07 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV08 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV09 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV10 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV11 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV12 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV13 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV14 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV15 => GBReg::new(0x00, 0b11111111, 0b11111111),

//         MMIO::PCM12 => match model { 
//             Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
//             Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000), // TODO: Verify startup value
//         },
//         MMIO::PCM34 => match model { 
//             Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
//             Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000), // TODO: Verify startup value
//         },