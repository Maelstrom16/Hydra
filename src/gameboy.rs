mod cpu;
mod memory;

use crate::common::{emulator::Emulator, errors::HydraIOError};
use std::fs;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Model {
    DMG0,
    DMG,
    MGB,
    SGB,
    SGB2,
    CGBdmg,
    AGBdmg,
    CGB,
    AGB,
}

pub struct GameBoy {
    //apu: apu::APU,
    cpu: cpu::CPU,
    memory: memory::Memory,
    //ppu: ppu:PPU
}

impl GameBoy {
    pub fn from_rom(rom: &Vec<u8>, model: Option<Model>) -> Self {
        let model = model.unwrap_or_else(|| {
            panic!();
        });
        GameBoy {
            cpu: cpu::CPU::default(),
            memory: memory::Memory::from_model(model).unwrap(),
        }
    }
    pub fn load_rom(&self) -> () {}
    // pub fn test() -> () {
    //     let mem_result = memory::Memory::from_file("testrom.gb");
    //     match mem_result {
    //         Ok(mut memory) => {
    //             println!("{:#x}", memory.read_u8(0x00D5));
    //         }
    //         Err(e) => {
    //             println!("Error loading cartridge: {}", e);
    //         }
    //     }
    // }

    pub const fn get_valid_extensions() -> &'static [&'static str] {
        &["gb", "gbc"]
    }
}

impl Emulator for GameBoy {
    fn launch() {
        println!("Launching Wyrm");
    }
}
