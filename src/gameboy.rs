mod cpu;
mod memory;
mod ppu;

use winit::window::Window;

use crate::{
    common::{
        emulator::{EmuMessage, Emulator},
        errors::HydraIOError,
    },
    config::Config,
    gameboy::memory::io::IO,
    graphics::Graphics,
};
use std::{
    ffi::OsStr,
    fmt, fs,
    path::Path,
    sync::{
        Arc, Barrier, Condvar, Mutex, RwLock, Weak,
        mpsc::{Receiver, Sender, channel},
    },
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
    cpu: cpu::CPU,
    memory: memory::Memory,
    ppu: ppu::PPU,
    clock: u32,

    channel: Receiver<EmuMessage>,
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>) -> Result<Sender<EmuMessage>, HydraIOError> {
        let rom = fs::read(path)?.into_boxed_slice();
        let (send, recv) = channel();

        thread::spawn(move || {
            let io = IO::new(model);
            let cpu = cpu::CPU::new(&rom, &io, model);
            let ppu = ppu::PPU::new(&io, window, graphics);
            let memory = memory::Memory::from_rom_and_model(rom, model, io).unwrap(); // TODO: Error should be handled rather than unwrapped
            GameBoy {
                cpu,
                memory,
                ppu,
                clock: 0,
                channel: recv,
            }
            .main_thread();
        });
        Ok(send)
    }
    pub fn from_model(path: &Path, model: Model, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>, config: &Config) -> Result<Sender<EmuMessage>, HydraIOError> {
        // If file extension is valid for the given model, initialize the emulator
        // Otherwise, return an InvalidExtension error
        match (path.extension().and_then(OsStr::to_str), model) {
            (Some("gb") | Some("gbc"), Model::GameBoy(revision)) => Ok(Self::from_revision(path, Model::GameBoy(Some(revision.unwrap_or(config.gb.default_models.dmg))), window, graphics)?),
            (Some("gb") | Some("gbc"), Model::SuperGameBoy(revision)) => Ok(Self::from_revision(
                path,
                Model::SuperGameBoy(Some(revision.unwrap_or(config.gb.default_models.sgb))),
                window,
                graphics,
            )?),
            (Some("gb") | Some("gbc"), Model::GameBoyColor(revision)) => Ok(Self::from_revision(
                path,
                Model::GameBoyColor(Some(revision.unwrap_or(config.gb.default_models.cgb))),
                window,
                graphics,
            )?),
            (Some("gb") | Some("gbc") | Some("gba"), Model::GameBoyAdvance(revision)) => Ok(Self::from_revision(
                path,
                Model::GameBoyAdvance(Some(revision.unwrap_or(config.gb.default_models.agb))),
                window,
                graphics,
            )?),
            (ext, model) => Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string))),
        }
    }
    fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom: Box<[u8]> = fs::read(path)?.into_boxed_slice();
        self.memory.hot_swap_rom(rom)
    }
    fn dump_mem(&self) {
        for y in 0..=0xFFF {
            print!("{:#06X}:   ", y << 4);
            for x in 0..=0xF {
                print!("{:02X} ", self.memory.read_u8(x | (y << 4)));
            }
            println!("");
        }
    }
}

const CYCLES_PER_FRAME: u32 = 70224;

impl Emulator for GameBoy {
    fn main_thread(mut self) {
        println!("Launching Wyrm");
        // Main loop
        loop {
            self.clock = (self.clock + 1) % CYCLES_PER_FRAME;
            self.cpu.step(&mut self.memory);
            self.ppu.step(&self.clock);
            // if self.clock % 456 == 0 {
            //     self.dump_mem();
            // }
        }
        println!("Exiting Wyrm");

        // Dump memory (for debugging)
        self.dump_mem();
    }
}
