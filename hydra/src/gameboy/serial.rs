use std::rc::Rc;

use crate::{common::{errors::HydraIOError, timing::ModuloCounter}, deserialize, gameboy::{GbMode, Model, interrupt::{Interrupt, InterruptFlags}, memory::MemoryMapped}, serialize};

pub struct SerialConnection {
    mode: Rc<GbMode>,

    m_cycle_counter: ModuloCounter<u8>,
    transfer_cycles_remaining: u8,
    local_clock: bool,

    data: u8,
}

impl SerialConnection {
    const DMG_LOW_SPEED: u8 = 128;
    const CGB_HIGH_SPEED: u8 = 4;

    pub fn new(mode: Rc<GbMode>) -> Self {
        let is_cgb = matches!(*mode, GbMode::CGB);
        SerialConnection { 
            mode, 

            m_cycle_counter: ModuloCounter::new(0, if is_cgb {Self::CGB_HIGH_SPEED} else {Self::DMG_LOW_SPEED}),
            transfer_cycles_remaining: 0, 
            local_clock: is_cgb,

            data: 0x00,
        }
    }

    pub fn tick(&mut self, interrupt_flags: &mut InterruptFlags) {
        if self.transfer_cycles_remaining > 0 && self.m_cycle_counter.increment() {
            self.data <<= 1;
            self.data |= 1; // TODO: replace with incoming bit once link play is implemented
            self.transfer_cycles_remaining -= 1;
            if self.transfer_cycles_remaining == 0 {
                interrupt_flags.request(Interrupt::Serial);
            }
        }
    }
}

impl SerialConnection {
    pub fn read_sb(&self) -> u8 {
        self.data
    }

    pub fn write_sb(&mut self, val: u8) {
        self.data = val
    }

    pub fn read_sc(&self) -> u8 {
        serialize!(
            ((self.transfer_cycles_remaining != 0) as u8) =>> [7];
            0b01111100;
            ((matches!(*self.mode, GbMode::DMG) || self.m_cycle_counter.modulus == Self::CGB_HIGH_SPEED) as u8) =>> [1];
            (self.local_clock as u8) =>> [0];
        )
    }

    pub fn write_sc(&mut self, val: u8) {
        deserialize!(val;
            [7] as bool =>> transfer_enabled;
            [1] as bool =>> high_speed;
            [0] as bool =>> (self.local_clock);
        );
        if self.transfer_cycles_remaining == 0 {
            if transfer_enabled {
                self.transfer_cycles_remaining = 8;
            }
            self.m_cycle_counter.modulus = if matches!(*self.mode, GbMode::CGB) && high_speed {Self::CGB_HIGH_SPEED} else {Self::DMG_LOW_SPEED};
            self.m_cycle_counter.reset();
        }
    }
}

impl MemoryMapped for SerialConnection {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF01 => Ok(self.read_sb()),
            0xFF02 => Ok(self.read_sc()),
            _ => Err(HydraIOError::OpenBusAccess),
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF01 => Ok(self.write_sb(val)),
            0xFF02 => Ok(self.write_sc(val)),
            _ => Err(HydraIOError::OpenBusAccess),
        }
    }
}