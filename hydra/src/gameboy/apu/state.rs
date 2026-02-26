use crate::{common::errors::HydraIOError, gameboy::{apu::channel::{Noise, Pulse, PulseType, Wave}, memory::MemoryMapped}};

pub struct ApuState {
    div: u8,

    pub(super) pulse1: Pulse,
    pub(super) pulse2: Pulse,
    pub(super) wave: Wave,
    pub(super) noise: Noise,
}

impl ApuState {
    pub fn new() -> Self {
        ApuState {
            div: 0,

            pulse1: Pulse::new(PulseType::Pulse1),
            pulse2: Pulse::new(PulseType::Pulse2),
            wave: Wave::new(),
            noise: Noise::new(),
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
            
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}