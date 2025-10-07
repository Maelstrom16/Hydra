mod cpu;
mod memory;
mod ppu;

use crate::{
    common::{emulator::Emulator, errors::HydraIOError},
    config::Config,
};
use std::{ffi::OsStr, fmt, fs, path::Path};

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
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model) -> Result<Self, HydraIOError> {
        let rom = fs::read(path)?.into_boxed_slice();

        Ok(GameBoy {
            cpu: cpu::CPU::from_rom_and_model(&rom, model),
            memory: memory::Memory::from_rom_and_model(rom, model)?,
            ppu: ppu::PPU {},
        })
    }
    pub fn from_model(path: &Path, model: Model, config: &Config) -> Result<Self, HydraIOError> {
        // If file extension is valid for the given model, initialize the emulator
        // Otherwise, return an InvalidExtension error
        match (path.extension().and_then(OsStr::to_str), model) {
            (Some("gb") | Some("gbc"), Model::GameBoy(revision)) => Ok(Self::from_revision(path, Model::GameBoy(Some(revision.unwrap_or(config.gb.default_models.dmg))))?),
            (Some("gb") | Some("gbc"), Model::SuperGameBoy(revision)) => Ok(Self::from_revision(path, Model::SuperGameBoy(Some(revision.unwrap_or(config.gb.default_models.sgb))))?),
            (Some("gb") | Some("gbc"), Model::GameBoyColor(revision)) => Ok(Self::from_revision(path, Model::GameBoyColor(Some(revision.unwrap_or(config.gb.default_models.cgb))))?),
            (Some("gb") | Some("gbc") | Some("gba"), Model::GameBoyAdvance(revision)) => Ok(Self::from_revision(path, Model::GameBoyAdvance(Some(revision.unwrap_or(config.gb.default_models.agb))))?),
            (ext, model) => Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string))),
        }
    }
    fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom: Box<[u8]> = fs::read(path)?.into_boxed_slice();
        self.memory.hot_swap_rom(rom)?;
        Ok(())
    }
}

impl Emulator for GameBoy {
    fn launch(&self) {
        println!("Launching Wyrm");
    }
}
