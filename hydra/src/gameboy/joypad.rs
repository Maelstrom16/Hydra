use std::{cell::RefCell, rc::Rc, sync::{Arc, RwLock}};

use sdl3::gamepad::Button;

use crate::{common::{bit::{BitVec, MaskedBitVec}, errors::HydraIOError}, gameboy::{Model, interrupt::{Interrupt, InterruptFlags}, memory::{MemoryMap, MemoryMapped}}, gamepad::{ControllerState, Direction, SdlContainer}};

pub struct Joypad {
    pub keyboard_vecs: InputVectors,
    controller_vecs: InputVectors,
    controllers: Arc<RwLock<ControllerState>>,
    joyp: MaskedBitVec<u8, true>,
}

impl Joypad {
    pub fn new(model: &Rc<Model>, controllers: Arc<RwLock<ControllerState>>) -> Self {
        Joypad { 
            keyboard_vecs: InputVectors::new(),
            controller_vecs: InputVectors::new(),
            controllers,
            joyp: MaskedBitVec::new(match model.is_monochrome() {
                true => 0xCF,
                false => 0xFF,
            }, 0b00111111, 0b00110000),
        }
    }

    fn is_polling_buttons(&self) -> bool {
        !self.joyp.test_bit(5)
    }
    
    fn is_polling_dpad(&self) -> bool {
        !self.joyp.test_bit(4)
    }

    fn refresh(&mut self, interrupt_flags: &mut InterruptFlags) {
        let mut after = 0b0000;
        if self.is_polling_buttons() {after |= self.keyboard_vecs.button_vector | self.controller_vecs.button_vector}
        if self.is_polling_dpad() {after |= self.keyboard_vecs.dpad_vector | self.controller_vecs.dpad_vector}
        
        if *self.joyp & after != 0 {
            interrupt_flags.request(Interrupt::Joypad);
        }

        *self.joyp = (*self.joyp & 0b00110000) | (after ^ 0b1111);
    }

    pub fn is_input_active(&self) -> bool {
        !self.joyp.read() & 0xF != 0
    }

    pub fn update_controller_vecs(&mut self, interrupt_flags: &mut InterruptFlags) {
        {
            let controllers = self.controllers.read().unwrap();
            self.controller_vecs.press_button(JoypButton::A, controllers.poll_button(Button::East) | controllers.poll_button(Button::West));
            self.controller_vecs.press_button(JoypButton::B, controllers.poll_button(Button::North) | controllers.poll_button(Button::South));
            self.controller_vecs.press_button(JoypButton::Start, controllers.poll_button(Button::Start));
            self.controller_vecs.press_button(JoypButton::Select, controllers.poll_button(Button::Back));
            self.controller_vecs.press_dpad(JoypDpad::Up, controllers.poll_direction(Direction::Up));
            self.controller_vecs.press_dpad(JoypDpad::Down, controllers.poll_direction(Direction::Down));
            self.controller_vecs.press_dpad(JoypDpad::Left, controllers.poll_direction(Direction::Left));
            self.controller_vecs.press_dpad(JoypDpad::Right, controllers.poll_direction(Direction::Right));
        }
                
        self.refresh(interrupt_flags);
    }
}

impl Joypad {   
    pub fn read_joyp(&self) -> u8 {
        self.joyp.read()
    }

    pub fn write_joyp(&mut self, val: u8, interrupt_flags: &mut InterruptFlags) {
        self.joyp.write(val);
        self.refresh(interrupt_flags);
    }
}

#[repr(u8)]
pub enum JoypDpad {
    Right = 0b00000001,
    Left  = 0b00000010,
    Up    = 0b00000100,
    Down  = 0b00001000,
}

#[repr(u8)]
pub enum JoypButton {
    A      = 0b00000001,
    B      = 0b00000010,
    Select = 0b00000100,
    Start  = 0b00001000,
}

pub struct InputVectors {
    button_vector: u8,
    dpad_vector: u8,
}

impl InputVectors {
    pub fn new() -> Self {
        InputVectors { 
            button_vector: 0b0000,
            dpad_vector: 0b0000, 
        }
    }

    pub fn press_button(&mut self, button: JoypButton, is_pressed: bool) {
        self.button_vector.map_bits(button as u8, is_pressed);
    }

    pub fn press_dpad(&mut self, dpad: JoypDpad, is_pressed: bool) {
        self.dpad_vector.map_bits(dpad as u8, is_pressed);
    }
}