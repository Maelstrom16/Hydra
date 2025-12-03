use std::{
    ffi::OsStr,
    path::Path,
    sync::{Arc, RwLock, mpsc::Sender},
};

use winit::window::Window;

use crate::{common::errors::HydraIOError, config::Config, gameboy, graphics::Graphics, window::HydraApp};

pub trait Emulator {
    fn main_thread(self);
}

pub fn init_from_file(path: &Path, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("gb") => gameboy::GameBoy::from_model(path, gameboy::Model::GameBoy(None), app),
        Some("gbc") => gameboy::GameBoy::from_model(path, gameboy::Model::GameBoyColor(None), app),
        ext => Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
    }
}

pub enum EmuMessage {
    Start,
    Stop,
    HotSwap(&'static Path),
}
