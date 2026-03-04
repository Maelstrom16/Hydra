use std::rc::Rc;

use crate::{common::errors::HydraIOError, deserialize, gameboy::{GbMode, Model, memory::MemoryMapped}, serialize};

pub struct SerialConnection {
    mode: Rc<GbMode>,

    transfer_enabled: bool,
    high_speed: bool,
    local_clock: bool,

    data: u8,
}

impl SerialConnection {
    pub fn new(mode: Rc<GbMode>) -> Self {
        SerialConnection { 
            transfer_enabled: false, 
            high_speed: matches!(*mode, GbMode::CGB), 
            local_clock: matches!(*mode, GbMode::CGB),

            data: 0x00,

            mode, 
        }
    }

    pub fn read_sb(&self) -> u8 {
        self.data
    }

    pub fn write_sb(&mut self, val: u8) {
        self.data = val
    }

    pub fn read_sc(&self) -> u8 {
        serialize!(
            (self.transfer_enabled as u8) =>> [7];
            0b01111100;
            ((matches!(*self.mode, GbMode::DMG) || self.high_speed) as u8) =>> [1];
            (self.local_clock as u8) =>> [0];
        )
    }

    pub fn write_sc(&mut self, val: u8) {
        deserialize!(val;
            [7] as bool =>> (self.transfer_enabled);
            [1] as bool =>> high_speed;
            [0] as bool =>> (self.local_clock);
        );
        self.high_speed = matches!(*self.mode, GbMode::CGB) && high_speed;
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