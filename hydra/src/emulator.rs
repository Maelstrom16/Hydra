pub mod gameboy;
pub mod gba;
pub mod n3ds;
pub mod nds;

use std::{
    ffi::OsStr, path::{Path, PathBuf}, sync::{Arc, RwLock, mpsc::Sender}, unimplemented,
};

use winit::{event::KeyEvent, window::Window};

use crate::{common::errors::HydraIOError, config::Config, emulator::gba::GbaTarget, graphics::Graphics, window::HydraApp};

const GB_FILE_FILTER: (&str, &[&str]) = ("Game Boy (Color)", &["gb", "gbc"]);
const GBA_FILE_FILTER: (&str, &[&str]) = ("Game Boy Advance", &["gba"]);
const NDS_FILE_FILTER: (&str, &[&str]) = ("Nintendo DS", &["nds", "srl"]);
const N3DS_FILE_FILTER: (&str, &[&str]) = ("Nintendo 3DS", &["3ds", "cci"]);

const ALL_FILE_FILTER: (&str, &[&str]) = ("All supported ROMs", &["gb", "gbc", "gba", "nds", "srl", "3ds", "cci"]);

pub trait Emulator {
    const CONSOLE_NAME: &str;
    const CORE_NAME: &str;
    const FILE_FILTERS: &'static [(&str, &[&str])];

    type Model;

    fn main_thread(self);
    fn rom_path(&self) -> &Path;
    fn try_init(model: Self::Model, rom_path: &PathBuf, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError>;

    fn save_path(&self) -> PathBuf { 
        let mut rom_path = self.rom_path().to_owned();
        rom_path.add_extension("hysav");
        rom_path
    }

    fn state_path(&self, slot: usize) -> PathBuf {
        let mut rom_path = self.rom_path().to_owned();
        rom_path.add_extension(&("hysav.st".to_owned() + &slot.to_string()));
        rom_path
    }
}

/// A dummy emulator which is used to represent that a console has not yet been chosen.
pub struct AllEmulator;

impl Emulator for AllEmulator {
    const CONSOLE_NAME: &str = "";
    const CORE_NAME: &str = "Hydra";
    const FILE_FILTERS: &[(&str, &[&str])] = &[ALL_FILE_FILTER, GB_FILE_FILTER, GBA_FILE_FILTER, NDS_FILE_FILTER, N3DS_FILE_FILTER];

    type Model = ();
    
    fn main_thread(self) { unimplemented!() }
    fn rom_path(&self) -> &Path { unimplemented!() }
    fn try_init(_model: Self::Model, rom_path: &PathBuf, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        match rom_path.extension().and_then(OsStr::to_str) {
            Some("gb") => gameboy::GameBoy::try_init(gameboy::Model::GameBoy(app.get_config().gb.default_models.dmg), rom_path, app),
            Some("gbc") => gameboy::GameBoy::try_init(gameboy::Model::GameBoyColor(app.get_config().gb.default_models.cgb), rom_path, app),
            Some("gba") => gba::GameBoyAdvance::try_init(GbaTarget::Gba, rom_path, app),
            Some("nds" | "srl") => nds::Nds::try_init((), rom_path, app),
            Some("3ds" | "cci") => n3ds::N3ds::try_init((), rom_path, app),
            ext => Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
        }
    }
}

pub enum EmuMessage {
    Pause,
    Reset,
    Stop,
    SaveStateSlot(usize),
    LoadStateSlot(usize),
    KeyboardInput(KeyEvent),
    HotSwap(&'static Path),
}
