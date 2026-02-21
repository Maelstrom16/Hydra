use std::sync::Arc;

use cpal::{Sample, Stream};

use crate::{audio::Audio, common::{audio, bit::BitVec, timing::{DynamicModuloCounter, DynamicOverflowCounter, ModuloCounter, OverflowCounter, Resettable}}, deserialize, gameboy::memory::{MMIO, MemoryMappedIo}, serialize};

pub struct Pulse {
    enabled: bool,
    
    period_timer: DynamicModuloCounter<u16, u16, Resettable<u16>>,
    period_sweep_direction: Direction,
    period_sweep_timer: DynamicModuloCounter<u8, Resettable<u8>, u8>,
    period_sweep_step: u8,

    volume: Resettable<u8>,
    volume_sweep_timer: DynamicModuloCounter<u8, Resettable<u8>, u8>,
    volume_sweep_direction: Resettable<Direction>,

    duty_index: usize,
    wavetable_index: ModuloCounter<usize>,

    length_timer: ModuloCounter<u8>,
    length_timer_enabled: bool,
}

impl Pulse {
    const WAVETABLES: [[u8; 8]; 4] = [
        [0x0, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0x0, 0x0, 0xF, 0xF, 0xF, 0xF],
        [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xF, 0xF],
    ];

    pub(super) fn new(pulse_type: PulseType) -> Self {
        let mut volume: Resettable<u8> = 0.into();
        volume.reset_value = match pulse_type {
            PulseType::Pulse1 => 0b1111,
            PulseType::Pulse2 => 0b0000,
        };

        Pulse { 
            enabled: false, 

            period_timer: DynamicModuloCounter::with_reset_value(0, 0x800, 0b11111111111.into()),
            period_sweep_direction: Direction::Decreasing, 
            period_sweep_timer: DynamicModuloCounter::new(0, 0.into()),
            period_sweep_step: 0, 

            volume,
            volume_sweep_timer: DynamicModuloCounter::new(0, match pulse_type {
                PulseType::Pulse1 => 3.into(),
                PulseType::Pulse2 => 0.into(),
            }),
            volume_sweep_direction: Direction::Decreasing.into(), 
            
            duty_index: match pulse_type {
                PulseType::Pulse1 => 2,
                PulseType::Pulse2 => 0,
            }, 
            wavetable_index: ModuloCounter::new(0, 8), 

            length_timer: ModuloCounter::new(0b111111, 64), 
            length_timer_enabled: false
        }
    }

    pub fn tick_and_sample(&mut self) -> f32 {
        if self.enabled {
            if self.period_timer.increment() && self.wavetable_index.increment() {
                self.period_timer.reset_value.reset();
            }

            let digital = Self::WAVETABLES[self.duty_index][self.wavetable_index.value];
            let analog = (digital * 0x11).to_sample::<f32>() * -1.0;
            analog.mul_amp(self.volume.current as f32 / 0xF as f32)
        } else {
            Sample::EQUILIBRIUM
        }
    }

    pub fn envelope_sweep(&mut self) {
        if self.volume_sweep_timer.increment() {
            self.volume.current = (self.volume.current.saturating_add_signed(self.volume_sweep_direction.current as i8)).min(0xF)
        }
    }

    pub fn period_sweep(&mut self) {
        if self.period_sweep_timer.increment() {
            let addend = (self.volume_sweep_direction.current as i16) * ((self.period_timer.reset_value.reset_value) / (2u16.pow(self.period_sweep_step as u32))) as i16;
            let new_period = self.period_timer.reset_value.reset_value.saturating_add_signed(addend);
            if new_period > 0x7FF {
                self.enabled = false;
            } else {
                self.period_timer.reset_value.reset_value = new_period;
            }
            self.period_sweep_timer.modulus.reset();
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_timer_enabled && self.length_timer.increment() {
            self.enabled = false;
        }
    }
}

impl MemoryMappedIo<{MMIO::NR10 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            (self.period_sweep_timer.modulus.reset_value) =>> 6..=4;
            (<Direction as Into<u8>>::into(self.period_sweep_direction)) =>> 3;
            (self.period_sweep_step) =>> 2..=0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            6..=4 =>> period_sweep_interval;
            3 as bool =>> period_sweep_direction;
            2..=0 =>> (self.period_sweep_step);
        );
        self.period_sweep_timer.modulus.reset_value = period_sweep_interval;
        self.period_sweep_timer.modulus.reset();
        self.period_sweep_direction = if period_sweep_direction {Direction::Increasing} else {Direction::Decreasing};
    }
}

impl MemoryMappedIo<{MMIO::NR11 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            (self.duty_index as u8) =>> 7..=6;
            0b00111111;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7..=6 =>> duty_index;
            5..=0 =>> initial_length_timer;
        );
        self.duty_index = duty_index as usize;
        self.length_timer.reset_value = initial_length_timer;
    }
}

impl MemoryMappedIo<{MMIO::NR12 as u16}> for Pulse {
    fn read(&self) -> u8 {
        serialize!(
            (self.volume.reset_value) =>> 7..=4;
            (<Direction as Into<u8>>::into(self.volume_sweep_direction.reset_value)) =>> 3;
            (self.volume_sweep_timer.modulus.reset_value) =>> 2..=0;
        )
    }

    fn write(&mut self, val: u8) {
        deserialize!(val;
            7..=4 =>> volume;
            3 as bool =>> volume_sweep_direction;
            2..=0 =>> volume_sweep_interval;
        );

        self.volume.reset_value = volume;
        self.volume_sweep_direction.reset_value = if volume_sweep_direction {Direction::Increasing} else {Direction::Decreasing};
        self.volume_sweep_timer.modulus.reset_value = volume_sweep_interval;

        if volume == 0 && !volume_sweep_direction {
            self.enabled = false;
        }
    }
}

impl MemoryMappedIo<{MMIO::NR13 as u16}> for Pulse {
    fn read(&self) -> u8 {
        0b11111111 // Write-only
    }

    fn write(&mut self, val: u8) {
        self.period_timer.reset_value.reset_value = (self.period_timer.reset_value.reset_value & 0b11100000000) | val as u16
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
            7 as bool =>> trigger;
            6 as bool =>> (self.length_timer_enabled);
            2..=0 =>> period_high;
        );

        self.period_timer.reset_value.reset_value = ((period_high as u16) << 8) | (self.period_timer.reset_value.reset_value & 0b00011111111);
        
        if trigger {
            self.enabled = true;
            self.length_timer.reset();
            self.period_timer.reset();
            self.volume.reset();
            self.volume_sweep_direction.reset();
            self.volume_sweep_timer.modulus.reset();
            self.volume_sweep_timer.reset();
            self.period_timer.reset_value.reset();
            self.period_sweep_timer.reset();
        }
    }
}

pub(super) enum PulseType {
    Pulse1,
    Pulse2
}

#[derive(Copy, Clone)]
#[repr(i8)]
enum Direction {
    Increasing = 1,
    Decreasing = -1
}

impl From<bool> for Direction {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Increasing,
            false => Self::Decreasing
        }
    }
}

impl Into<u8> for Direction {
    fn into(self) -> u8 {
        match self {
            Self::Increasing => 1,
            Self::Decreasing => 0
        }
    }
}



pub struct Wave {
    dac_enabled: bool,
    enabled: bool,

    period_timer: DynamicModuloCounter<u16, u16, Resettable<u16>>,

    volume: u8,

    wavetable: [u8; 32],
    wavetable_index: ModuloCounter<usize>,

    length_timer: OverflowCounter<u8>,
    length_timer_enabled: bool,
}

impl Wave {
    pub(super) fn new() -> Self {
        Wave { 
            dac_enabled: false,
            enabled: false,

            period_timer: DynamicModuloCounter::with_reset_value(0, 0x800, 0b11111111111.into()),

            volume: 0b00,

            wavetable: [0x00; 32], 
            wavetable_index: ModuloCounter::new(0, 32), 

            length_timer: OverflowCounter::new(0b11111111), 
            length_timer_enabled: false
        }
    }

    const VOLUME_SHIFT_TABLE: [u8; 4] = [4, 0, 1, 2];

    pub fn tick_and_sample(&mut self) -> f32 {
        if self.dac_enabled && self.enabled {
            if self.period_timer.increment() {
                self.wavetable_index.increment();
            }
            self.period_timer.reset_value.reset();
            ((self.wavetable[self.wavetable_index.value] >> Self::VOLUME_SHIFT_TABLE[self.volume as usize]) * 0x11).to_sample::<f32>()
        } else {
            Sample::EQUILIBRIUM
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_timer_enabled && self.length_timer.increment() {
            self.enabled = false;
        }
    }
}

impl Wave {
    pub fn read_nr30(&self) -> u8 {
        serialize!(
            (self.dac_enabled as u8) =>> 7;
            0b01111111;
        )
    }

    pub fn write_nr30(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> (self.dac_enabled);
        );
    }

    pub fn read_nr31(&self) -> u8 {
        0b11111111 // Write-only
    }

    pub fn write_nr31(&mut self, val: u8) {
        self.length_timer.reset_value = val
    }

    pub fn read_nr32(&self) -> u8 {
        serialize!(
            (self.volume) =>> 6..=5;
        )
    }

    pub fn write_nr32(&mut self, val: u8) {
        deserialize!(val;
            6..=5 =>> (self.volume);
        );
    }

    pub fn read_nr33(&self) -> u8 {
        0b11111111 // Write-only
    }

    pub fn write_nr33(&mut self, val: u8) {
        self.period_timer.reset_value.reset_value = (self.period_timer.reset_value.reset_value & 0b11100000000) | val as u16
    }
    
    pub fn read_nr34(&self) -> u8 {
        serialize!(
            0b10111111;
            (self.length_timer_enabled as u8) =>> 6;
        )
    }

    pub fn write_nr34(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> trigger;
            6 as bool =>> (self.length_timer_enabled);
            2..=0 =>> period_high;
        );

        self.period_timer.reset_value.reset_value = ((period_high as u16) << 8) | (self.period_timer.reset_value.reset_value & 0b00011111111);
        
        if trigger {
            self.enabled = true;
            self.length_timer.reset();
            self.period_timer.reset();
        }
    }

    pub fn read_waveram(&self, address: usize) -> u8 {
        let index = address * 2;
        self.wavetable[index] << 4 | self.wavetable[index + 1]
    }

    pub fn write_waveram(&mut self, val: u8, address: usize) {
        let index = address * 2;
        deserialize!(val;
            7..=4 =>> wave_high;
            3..=0 =>> wave_low;
        );
        self.wavetable[index] = wave_high;
        self.wavetable[index + 1] = wave_low;
    }
}

pub struct Noise {
    enabled: bool,

    volume: Resettable<u8>,
    volume_sweep_timer: DynamicModuloCounter<u8, Resettable<u8>, u8>,
    volume_sweep_direction: Resettable<Direction>,

    initial_shift: u8,
    initial_divider: u8,
    lfsr_timer: DynamicModuloCounter<u32, Resettable<u32>, u32>,
    lfsr: u16,
    use_bit_7: bool,

    length_timer: ModuloCounter<u8>,
    length_timer_enabled: bool,

    shifted_out: bool,
}

impl Noise {
    pub(super) fn new() -> Self {
        Noise { 
            enabled: false,

            volume: 0b0000.into(),
            volume_sweep_timer: DynamicModuloCounter::new(0, 0.into()),
            volume_sweep_direction: Direction::Decreasing.into(), 

            initial_divider: 0,
            initial_shift: 0,
            lfsr_timer: DynamicModuloCounter::new(0, 1.into()),
            lfsr: 0,
            use_bit_7: false,

            length_timer: ModuloCounter::new(0b111111, 64), 
            length_timer_enabled: false,

            shifted_out: false,
        }
    }

    pub fn tick_and_sample(&mut self) -> f32 {
        if self.enabled {
            if self.lfsr_timer.increment() {
                self.lfsr_timer.modulus.reset();
                let bit0 = self.lfsr.test_bit(0);
                let xnor = !(bit0 ^ self.lfsr.test_bit(1));
                self.lfsr.map_bit(15, xnor);
                if self.use_bit_7 {
                    self.lfsr.map_bit(7, xnor);
                }
                self.lfsr >>= 1;
                self.shifted_out = bit0;
            }
            (!self.shifted_out as u8 * self.volume.current * 0x11).to_sample::<f32>()
        } else {
            Sample::EQUILIBRIUM
        }
    }

    pub fn envelope_sweep(&mut self) {
        if self.volume_sweep_timer.increment() {
            self.volume.current = (self.volume.current.saturating_add_signed(self.volume_sweep_direction.current as i8)).min(0xF)
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_timer_enabled && self.length_timer.increment() {
            self.enabled = false;
        }
    }
}

impl Noise {
    pub fn read_nr41(&self) -> u8 {
        0b11111111 // Write-only
    }

    pub fn write_nr41(&mut self, val: u8) {
        deserialize!(val;
            5..=0 =>> (self.length_timer.reset_value);
        );
    }
    
    pub fn read_nr42(&self) -> u8 {
        serialize!(
            (self.volume.reset_value) =>> 7..=4;
            (<Direction as Into<u8>>::into(self.volume_sweep_direction.reset_value)) =>> 3;
            (self.volume_sweep_timer.modulus.reset_value) =>> 2..=0;
        )
    }

    pub fn write_nr42(&mut self, val: u8) {
        deserialize!(val;
            7..=4 =>> volume;
            3 as bool =>> volume_sweep_direction;
            2..=0 =>> volume_sweep_interval;
        );

        self.volume.reset_value = volume;
        self.volume_sweep_direction.reset_value = if volume_sweep_direction {Direction::Increasing} else {Direction::Decreasing};
        self.volume_sweep_timer.modulus.reset_value = volume_sweep_interval;

        if volume == 0 && !volume_sweep_direction {
            self.enabled = false;
        }
    }
    
    pub fn read_nr43(&self) -> u8 {
        serialize!(
            (self.initial_shift) =>> 7..=4;
            (self.use_bit_7 as u8) =>> 3;
            (self.initial_divider) =>> 2..=0;
        )
    }

    pub fn write_nr43(&mut self, val: u8) {
        deserialize!(val;
            7..=4 =>> (self.initial_shift);
            3 as bool =>> (self.use_bit_7);
            2..=0 =>> (self.initial_divider);
        );
        
        let divider = if self.initial_divider != 0 {self.initial_divider as u32 * 2} else {1};
        self.lfsr_timer.modulus.reset_value = divider * 2u32.pow(self.initial_shift as u32);
    }
    
    pub fn read_nr44(&self) -> u8 {
        serialize!(
            0b10111111;
            (self.length_timer_enabled as u8) =>> 6;
        )
    }

    pub fn write_nr44(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> trigger;
            6 as bool =>> (self.length_timer_enabled);
        );
        
        if trigger {
            self.enabled = true;
            self.length_timer.reset();
            self.volume.reset();
            self.volume_sweep_direction.reset();
            self.volume_sweep_timer.modulus.reset();
            self.volume_sweep_timer.reset();
            self.lfsr = 0;
            self.lfsr_timer.modulus.reset();
        }
    }
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