use std::rc::Rc;

use crate::{deserialize, gameboy::Model, serialize};

pub struct SerialConnection {
    model: Rc<Model>,

    transfer_enabled: bool,
    high_speed: bool,
    local_clock: bool,

    data: u8,
}

impl SerialConnection {
    pub fn new(model: Rc<Model>) -> Self {
        SerialConnection { 
            transfer_enabled: false, 
            high_speed: model.is_color(), 
            local_clock: model.is_monochrome(),

            data: 0x00,

            model, 
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
            (self.transfer_enabled as u8) =>> 7;
            ((self.model.is_monochrome() || self.high_speed) as u8) =>> 1;
            (self.local_clock as u8) =>> 0;
        )
    }

    pub fn write_sc(&mut self, val: u8) {
        deserialize!(val;
            7 as bool =>> (self.transfer_enabled);
            1 as bool =>> high_speed;
            0 as bool =>> (self.local_clock);
        );
        self.high_speed = self.model.is_color() && high_speed;
    }
}