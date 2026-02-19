use std::sync::Arc;

use cpal::Stream;

use crate::{audio::Audio, common::{audio, util::Delayed}, deserialize, gameboy::memory::{MMIO, MemoryMappedIo}, serialize};

pub struct Pulse {
    enabled: bool,
    
    period: u16,
    period_div: u16,
    period_sweep_interval: u8,
    period_sweep_direction: Direction,
    period_sweep_step: u8,

    volume: u8,
    volume_sweep_interval: u8,
    volume_sweep_direction: Direction,

    duty_index: Delayed<usize>,
    wavetable_index: usize,

    length_timer: u8,
    length_timer_enabled: bool,
}

impl Pulse {
    const WAVETABLES: [[u8; 8]; 4] = [
        [0x0, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0x0, 0x0, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xF, 0xF],
    ];

    pub fn new1() -> Self {
        Pulse { 
            enabled: true, 

            period: 0b11111111111, 
            period_div: 0,
            period_sweep_interval: 0, 
            period_sweep_direction: Direction::Decreasing, 
            period_sweep_step: 0, 

            volume: 0b1111, 
            volume_sweep_interval: 3, 
            volume_sweep_direction: Direction::Decreasing, 
            
            duty_index: 2.into(), 
            wavetable_index: 0, 

            length_timer: 0b111111, 
            length_timer_enabled: false
        }
    }

    pub fn new2() -> Self {
        Pulse { 
            enabled: true, 

            period: 0b11111111111,
            period_div: 0,
            period_sweep_interval: 0, 
            period_sweep_direction: Direction::Decreasing, 
            period_sweep_step: 0, 

            volume: 0, 
            volume_sweep_interval: 0, 
            volume_sweep_direction: Direction::Decreasing, 
            
            duty_index: 1.into(), 
            wavetable_index: 0, 
            
            length_timer: 0b111111, 
            length_timer_enabled: false 
        }
    }

    pub fn tick_and_sample(&mut self) -> u8 {
        if self.period_div >= 0x7FF {
            self.period_div = self.period;
            self.wavetable_index = (self.wavetable_index + 1) % 8;
            if self.wavetable_index == 0 {
                self.duty_index.process_queue();
            }
        } else {
            self.period_div += 1
        }

        Self::WAVETABLES[*self.duty_index][self.wavetable_index]
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
            7..=6 =>> duty_index;
            5..=0 =>> (self.length_timer);
        );
        self.duty_index.queue(duty_index as usize);
    }
}

impl MemoryMappedIo<{MMIO::NR12 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            (self.volume) =>> 7..=4;
            (self.volume_sweep_direction as u8) =>> 3;
            (self.volume_sweep_interval) =>> 2..=0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7..=4 =>> (self.volume);
            3 as bool =>> volume_sweep_direction;
            2..=0 =>> (self.volume_sweep_interval);
        );

        self.volume_sweep_direction = if volume_sweep_direction {Direction::Increasing} else {Direction::Decreasing};
    }
}

impl MemoryMappedIo<{MMIO::NR13 as u16}> for Pulse {
    fn read(&self) -> u8 {
        0b11111111 // Write-only
    }

    fn write(&mut self, val: u8) {
        self.period = (self.period & 0b11100000000) | val as u16
    }
}

impl MemoryMappedIo<{MMIO::NR14 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            0b10111111;
            (self.length_timer_enabled as u8) =>> 6;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> (self.enabled);
            6 as bool =>> (self.length_timer_enabled);
            2..=0 =>> period_high;
        );
        self.period = ((period_high as u16) << 8) | (self.period & 0b00011111111)
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Increasing = 1,
    Decreasing = 0
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