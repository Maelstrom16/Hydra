use crate::{common::{bit::MaskedBitVec, errors::HydraIOError}, gameboy::memory::MemoryMapped};

pub struct InterruptFlags {
    interrupts: MaskedBitVec<u8, true>
}

impl InterruptFlags {
    pub fn new() -> Self {
        InterruptFlags {
            interrupts: MaskedBitVec::new(0b11100001, 0b00011111, 0b00011111)
        }
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        *self.interrupts |= interrupt as u8
    }

    pub fn is_requested(&self, interrupt: Interrupt) -> bool {
        *self.interrupts & interrupt as u8 != 0
    }

    pub fn get_inner(&mut self) -> &mut MaskedBitVec<u8, true> {
        &mut self.interrupts
    }
    
    pub fn read_if(&self) -> u8 {
        self.interrupts.read()
    }

    pub fn write_if(&mut self, val: u8) {
        self.interrupts.write(val);
    }
}

impl MemoryMapped for InterruptFlags {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF0F => Ok(self.read_if()),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF0F => Ok(self.write_if(val)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

pub struct InterruptEnable {
    interrupts: MaskedBitVec<u8, false> // TODO: false assumed due to startup value -- verify this
}

impl InterruptEnable {
    pub fn new() -> Self {
        InterruptEnable {
            interrupts: MaskedBitVec::new(0b00000000, 0b00011111, 0b00011111)
        }
    }
    
    pub fn read_ie(&self) -> u8 {
        self.interrupts.read()
    }

    pub fn write_ie(&mut self, val: u8) {
        self.interrupts.write(val)
    }
}

impl MemoryMapped for InterruptEnable {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFFFF => Ok(self.read_ie()),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFFFF => Ok(self.write_ie(val)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

#[repr(u8)]
pub enum Interrupt {
    Vblank = 0b00000001,
    Stat   = 0b00000010,
    Timer  = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}