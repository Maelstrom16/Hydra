use std::{cell::RefCell, rc::Rc};

use crate::{common::{bit::{BitVec, MaskedBitVec}, errors::HydraIOError}, gameboy::{interrupt::{Interrupt, InterruptFlags}, memory::MemoryMapped}};

pub struct Joypad {
    button_vector: u8,
    dpad_vector: u8,
    joyp: MaskedBitVec<u8, true>,
    interrupt_flags: Rc<RefCell<InterruptFlags>>
}

impl Joypad {
    pub fn new(interrupt_flags: Rc<RefCell<InterruptFlags>>) -> Self {
        Joypad { 
            button_vector: 0b0000,
            dpad_vector: 0b0000,
            joyp: MaskedBitVec::new(0xCF, 0b00111111, 0b00110000),
            interrupt_flags 
        }
    }

    fn is_polling_buttons(&self) -> bool {
        !self.joyp.test_bit(5)
    }
    
    fn is_polling_dpad(&self) -> bool {
        !self.joyp.test_bit(4)
    }

    fn refresh(&mut self) {
        let mut after = 0b0000;
        if self.is_polling_buttons() {after |= self.button_vector}
        if self.is_polling_dpad() {after |= self.dpad_vector}
        
        if *self.joyp & after != 0 {
            self.interrupt_flags.borrow_mut().request(Interrupt::Joypad);
        }

        *self.joyp = (*self.joyp & 0b00110000) | (after ^ 0b1111);
    }

    pub fn press_button(&mut self, button: Button, is_pressed: bool) {
        self.button_vector.map_bits(button as u8, is_pressed);
        self.refresh();
    }

    pub fn press_dpad(&mut self, dpad: Dpad, is_pressed: bool) {
        self.dpad_vector.map_bits(dpad as u8, is_pressed);
        self.refresh();
    }

    pub fn is_input_active(&self) -> bool {
        !self.joyp.read() & 0xF != 0
    }
}

impl Joypad {   
    pub fn read_joyp(&self) -> u8 {
        self.joyp.read()
    }

    pub fn write_joyp(&mut self, val: u8) {
        self.joyp.write(val);
        self.refresh();
    }
}

impl MemoryMapped for Joypad {
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF00 => Ok(self.read_joyp()),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16) -> Result<(), HydraIOError> {
        match address {
            0xFF00 => Ok(self.write_joyp(val)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

#[repr(u8)]
pub enum Dpad {
    Right = 0b00000001,
    Left  = 0b00000010,
    Up    = 0b00000100,
    Down  = 0b00001000,
}

#[repr(u8)]
pub enum Button {
    A      = 0b00000001,
    B      = 0b00000010,
    Select = 0b00000100,
    Start  = 0b00001000,
}