use std::{
    ffi::OsStr,
    path::Path,
    sync::{Arc, RwLock, mpsc::Sender},
};

use winit::{event::KeyEvent, window::Window};

use crate::{common::errors::HydraIOError, config::Config, gameboy, graphics::Graphics, window::HydraApp};

pub trait Emulator {
    fn main_thread(self);
}

pub fn init_from_file(path: &Path, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("gb") => gameboy::GameBoy::new(path, gameboy::Model::GameBoy(app.get_config().gb.default_models.dmg), app),
        Some("gbc") => gameboy::GameBoy::new(path, gameboy::Model::GameBoyColor(app.get_config().gb.default_models.cgb), app),
        ext => Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
    }
}

pub enum EmuMessage {
    Start,
    Stop,
    KeyboardInput(KeyEvent),
    HotSwap(&'static Path),
}
