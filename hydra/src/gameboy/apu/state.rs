use std::rc::Rc;

use cpal::Sample;

use crate::{common::errors::HydraIOError, deserialize, gameboy::{Model, apu::channel::{Noise, Pulse, PulseType, Wave}, memory::MemoryMapped}, serialize};

pub struct ApuState {
    model: Rc<Model>,

    master_enable: bool,
    master_amp_l: u8,
    master_amp_r: u8,
    div: u8,

    pub(super) pulse1: Pulse,
    pub(super) pulse2: Pulse,
    pub(super) wave: Wave,
    pub(super) noise: Noise,

    vin_amp_l: u8,
    vin_amp_r: u8,

    prev_samples: [f32; 4],
    amplitudes_l: [u8; 4],
    amplitudes_r: [u8; 4],
}

impl ApuState {
    pub fn new(model: Rc<Model>) -> Self {
        ApuState {
            model, 

            master_enable: true,
            master_amp_l: 7,
            master_amp_r: 7,
            div: 0,

            pulse1: Pulse::new(PulseType::Pulse1),
            pulse2: Pulse::new(PulseType::Pulse2),
            wave: Wave::new(),
            noise: Noise::new(),

            vin_amp_l: 0,
            vin_amp_r: 0,

            prev_samples: [0.0; 4],
            amplitudes_l: [1; 4],
            amplitudes_r: [1, 1, 0, 0],
        }
    }

    /// Tick function to be called on every DIV-APU tick to update audio channel fields.
    pub fn apu_tick(&mut self) {
        self.div = self.div.wrapping_add(1);

        if self.div % 8 == 0 {
            self.pulse1.envelope_sweep();
            self.pulse2.envelope_sweep();
            self.noise.envelope_sweep();
        }

        if self.div % 4 == 0 {
            self.pulse1.period_sweep();
        }

        if self.div % 2 == 0 {
            self.pulse1.tick_length();
            self.pulse2.tick_length();
            self.wave.tick_length();
            self.noise.tick_length();
        }
    }

    pub fn dot_tick(&mut self, dot_counter: u8) -> [f32; 2] {
        self.prev_samples[2] = self.wave.tick_and_sample();
        if dot_counter % 4 == 0 {
            self.prev_samples[0] = self.pulse1.tick_and_sample();
            self.prev_samples[1] = self.pulse2.tick_and_sample();
            if dot_counter % 8 == 0 {
                self.prev_samples[3] = self.noise.tick_and_sample();
            }
        }

        [self.prev_samples.iter().enumerate().fold(0.0, |l, (index, sample)| l + sample.mul_amp(Self::amp_from_u1(self.amplitudes_l[index]))).mul_amp(Self::amp_from_u3(self.master_amp_l) / 4.0),
         self.prev_samples.iter().enumerate().fold(0.0, |r, (index, sample)| r + sample.mul_amp(Self::amp_from_u1(self.amplitudes_r[index]))).mul_amp(Self::amp_from_u3(self.master_amp_r) / 4.0)]
    }

    fn amp_from_u1(u1: u8) -> f32 {
        u1 as f32
    }

    fn amp_from_u3(u3: u8) -> f32 {
        // Maps 0 - 7 to 0.125 - 1.000
        (u3 + 1) as f32 / 8.0
    }
}

impl ApuState {
    fn read_nr50(&self) -> u8 {
        serialize!(
            (self.vin_amp_l) =>> [7];
            (self.master_amp_l) =>> [6..=4];
            (self.vin_amp_r) =>> [3];
            (self.master_amp_r) =>> [2..=0];
        )
    }

    fn write_nr50(&mut self, val: u8) {
        deserialize!(val;
            [7] =>> (self.vin_amp_l);
            [6..=4] =>> (self.master_amp_l);
            [3] =>> (self.vin_amp_r);
            [2..=0] =>> (self.master_amp_r);
        );
    }

    fn read_nr51(&self) -> u8 {
        serialize!(
            (self.amplitudes_l[3]) =>> [7];
            (self.amplitudes_l[2]) =>> [6];
            (self.amplitudes_l[1]) =>> [5];
            (self.amplitudes_l[0]) =>> [4];
            (self.amplitudes_r[3]) =>> [3];
            (self.amplitudes_r[2]) =>> [2];
            (self.amplitudes_r[1]) =>> [1];
            (self.amplitudes_r[0]) =>> [0];
        )
    }

    fn write_nr51(&mut self, val: u8) {
        deserialize!(val;
            [7] =>> (self.amplitudes_l[3]);
            [6] =>> (self.amplitudes_l[2]);
            [5] =>> (self.amplitudes_l[1]);
            [4] =>> (self.amplitudes_l[0]);
            [3] =>> (self.amplitudes_r[3]);
            [2] =>> (self.amplitudes_r[2]);
            [1] =>> (self.amplitudes_r[1]);
            [0] =>> (self.amplitudes_r[0]);
        );
    }

    fn read_nr52(&self) -> u8 {
        serialize!(
            (self.master_enable as u8) =>> [7];
            0b01110000;
            (self.noise.is_enabled() as u8) =>> [3];
            (self.wave.is_enabled() as u8) =>> [2];
            (self.pulse2.is_enabled() as u8) =>> [1];
            (self.pulse1.is_enabled() as u8) =>> [0];
        )
    }

    fn write_nr52(&mut self, val: u8) {
        deserialize!(val;
            [7] as bool =>> (self.master_enable);
        );
    }

    fn read_pcm12(&self) -> u8 {
        0x00 // TODO: Implement properly
    }

    fn read_pcm34(&self) -> u8 {
        0x00 // TODO: Implement properly
    }
}

impl MemoryMapped for ApuState {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF10 => Ok(self.pulse1.read_nr10()),
            0xFF11 => Ok(self.pulse1.read_nrx1()),
            0xFF12 => Ok(self.pulse1.read_nrx2()),
            0xFF13 => Ok(self.pulse1.read_nrx3()),
            0xFF14 => Ok(self.pulse1.read_nrx4()),

            0xFF16 => Ok(self.pulse2.read_nrx1()),
            0xFF17 => Ok(self.pulse2.read_nrx2()),
            0xFF18 => Ok(self.pulse2.read_nrx3()),
            0xFF19 => Ok(self.pulse2.read_nrx4()),

            0xFF1A => Ok(self.wave.read_nr30()),
            0xFF1B => Ok(self.wave.read_nr31()),
            0xFF1C => Ok(self.wave.read_nr32()),
            0xFF1D => Ok(self.wave.read_nr33()),
            0xFF1E => Ok(self.wave.read_nr34()),
            0xFF30..=0xFF3F => Ok(self.wave.read_waveram(address as usize - 0xFF30)),

            0xFF20 => Ok(self.noise.read_nr41()),
            0xFF21 => Ok(self.noise.read_nr42()),
            0xFF22 => Ok(self.noise.read_nr43()),
            0xFF23 => Ok(self.noise.read_nr44()),

            0xFF24 => Ok(self.read_nr50()),
            0xFF25 => Ok(self.read_nr51()),
            0xFF26 => Ok(self.read_nr52()),

            0xFF76 if self.model.is_color() => Ok(self.read_pcm12()),
            0xFF77 if self.model.is_color() => Ok(self.read_pcm34()),

            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF10 => Ok(self.pulse1.write_nr10(val)),
            0xFF11 => Ok(self.pulse1.write_nrx1(val)),
            0xFF12 => Ok(self.pulse1.write_nrx2(val)),
            0xFF13 => Ok(self.pulse1.write_nrx3(val)),
            0xFF14 => Ok(self.pulse1.write_nrx4(val)),

            0xFF16 => Ok(self.pulse2.write_nrx1(val)),
            0xFF17 => Ok(self.pulse2.write_nrx2(val)),
            0xFF18 => Ok(self.pulse2.write_nrx3(val)),
            0xFF19 => Ok(self.pulse2.write_nrx4(val)),

            0xFF1A => Ok(self.wave.write_nr30(val)),
            0xFF1B => Ok(self.wave.write_nr31(val)),
            0xFF1C => Ok(self.wave.write_nr32(val)),
            0xFF1D => Ok(self.wave.write_nr33(val)),
            0xFF1E => Ok(self.wave.write_nr34(val)),
            0xFF30..=0xFF3F => Ok(self.wave.write_waveram(val, address as usize - 0xFF30)),

            0xFF20 => Ok(self.noise.write_nr41(val)),
            0xFF21 => Ok(self.noise.write_nr42(val)),
            0xFF22 => Ok(self.noise.write_nr43(val)),
            0xFF23 => Ok(self.noise.write_nr44(val)),

            0xFF24 => Ok(self.write_nr50(val)),
            0xFF25 => Ok(self.write_nr51(val)),
            0xFF26 => Ok(self.write_nr52(val)),

            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}