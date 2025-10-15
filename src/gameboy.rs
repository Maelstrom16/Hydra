mod cpu;
mod memory;
mod ppu;

use winit::window::Window;

use crate::{
    common::{emulator::Emulator, errors::HydraIOError},
    config::Config, graphics::Graphics,
};
use std::{
    ffi::OsStr,
    fmt, fs,
    path::Path,
    sync::{Arc, Barrier, Condvar, Mutex, RwLock, Weak},
    thread,
};

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Model {
    // A model with no revision specifies a target console (i.e. any revision).
    GameBoy(Option<GBRevision>),
    SuperGameBoy(Option<SGBRevision>),
    GameBoyColor(Option<CGBRevision>),
    GameBoyAdvance(Option<AGBRevision>),
}
impl Model {
    const fn as_str(&self) -> &'static str {
        match self {
            Model::GameBoy(Some(GBRevision::DMG0)) => "Game Boy (DMG0)",
            Model::GameBoy(Some(GBRevision::DMG)) => "Game Boy (DMG)",
            Model::GameBoy(Some(GBRevision::MGB)) => "Game Boy Pocket",
            Model::GameBoy(Some(GBRevision::CGBdmg)) => "Game Boy Color (DMG compat mode)",
            Model::GameBoy(Some(GBRevision::AGBdmg)) => "Game Boy Advance (DMG compat mode)",
            Model::GameBoy(None) => "Game Boy",
            Model::SuperGameBoy(Some(SGBRevision::SGB)) => "Super Game Boy",
            Model::SuperGameBoy(Some(SGBRevision::SGB2)) => "Super Game Boy 2",
            Model::SuperGameBoy(None) => "Super Game Boy",
            Model::GameBoyColor(Some(CGBRevision::CGB0)) => "Game Boy Color (CGB0)",
            Model::GameBoyColor(Some(CGBRevision::CGB)) => "Game Boy Color (CGB)",
            Model::GameBoyColor(None) => "Game Boy Color",
            Model::GameBoyAdvance(Some(AGBRevision::AGB0)) => "Game Boy Advance (AGB0)",
            Model::GameBoyAdvance(Some(AGBRevision::AGB)) => "Game Boy Advance (AGB)",
            Model::GameBoyAdvance(None) => "Game Boy Advance",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GBRevision {
    DMG0,
    DMG,
    MGB,
    CGBdmg, // Special mode to specify Game Boy Color running original GB game in compatibility mode.
    AGBdmg, // Special mode to specify Game Boy Advance running original GB game in compatibility mode.
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SGBRevision {
    SGB,
    SGB2,
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CGBRevision {
    CGB0,
    CGB,
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AGBRevision {
    AGB0,
    AGB,
}

pub struct GameBoy {
    //apu: apu::APU,
    cpu: Mutex<Option<cpu::CPU>>,
    memory: Arc<RwLock<memory::Memory>>,
    ppu: Mutex<Option<ppu::PPU>>,
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>) -> Result<Self, HydraIOError> {
        let rom = fs::read(path)?.into_boxed_slice();
        let memory = Arc::new(RwLock::new(memory::Memory::from_rom_and_model(rom.clone(), model)?));
        let cpu = Mutex::new(Some(cpu::CPU::new(&rom, model, memory.clone())));
        let ppu = Mutex::new(Some(ppu::PPU::new(window, graphics, memory.clone())));

        Ok(GameBoy {
            cpu,
            memory,
            ppu,
        })
    }
    pub fn from_model(path: &Path, model: Model, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>, config: &Config) -> Result<Self, HydraIOError> {
        // If file extension is valid for the given model, initialize the emulator
        // Otherwise, return an InvalidExtension error
        match (path.extension().and_then(OsStr::to_str), model) {
            (Some("gb") | Some("gbc"), Model::GameBoy(revision)) => 
                Ok(Self::from_revision(path, Model::GameBoy(Some(revision.unwrap_or(config.gb.default_models.dmg))), window, graphics)?),
            (Some("gb") | Some("gbc"), Model::SuperGameBoy(revision)) => 
                Ok(Self::from_revision(path, Model::SuperGameBoy(Some(revision.unwrap_or(config.gb.default_models.sgb))), window, graphics)?),
            (Some("gb") | Some("gbc"), Model::GameBoyColor(revision)) => 
                Ok(Self::from_revision(path, Model::GameBoyColor(Some(revision.unwrap_or(config.gb.default_models.cgb))), window, graphics)?),
            (Some("gb") | Some("gbc") | Some("gba"), Model::GameBoyAdvance(revision)) => 
                Ok(Self::from_revision(path, Model::GameBoyAdvance(Some(revision.unwrap_or(config.gb.default_models.agb))), window, graphics)?),
            (ext, model) => 
                Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string))),
        }
    }
    fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom: Box<[u8]> = fs::read(path)?.into_boxed_slice();
        self.memory.write().unwrap().hot_swap_rom(rom)
    }
}

impl Emulator for GameBoy {
    fn main_thread(self: Arc<GameBoy>) {
        println!("Launching Wyrm");
        thread::spawn(move || {
            // Spawn child threads
            let mut cpu = self.cpu.lock().unwrap().take().unwrap();
            let mut ppu = self.ppu.lock().unwrap().take().unwrap();
            thread::spawn(|| println!("APU"));
            
            // Main thread
            loop {
                cpu.step();
                ppu.step();
            }
            println!("Exiting Wyrm");

            // Dump memory (for debugging)
            for y in 0..=0xFFF {
                print!("{:#06X}:   ", y<<4);
                for x in 0..=0xF {
                    print!("{:02X} ", self.memory.read().unwrap().read_u8(x | (y << 4)));
                }
                println!("");
            }
        });
    }
}
