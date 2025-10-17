use std::{ffi::OsStr, path::Path, sync::{mpsc::Sender, Arc, RwLock}};

use winit::window::Window;

use crate::{common::errors::HydraIOError, config::Config, gameboy, graphics::Graphics};

pub trait Emulator {
    fn main_thread(self);
}

pub fn init_from_file(path: &Path, window: Arc<Window>, graphics: Arc<RwLock<Graphics>>, config: &Config) -> Result<Sender<EmuMessage>, HydraIOError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("gb") => 
            gameboy::GameBoy::from_model(path, gameboy::Model::GameBoy(None), window, graphics, config),
        Some("gbc") => 
            gameboy::GameBoy::from_model(path, gameboy::Model::GameBoyColor(None), window, graphics, config),
        ext => 
            Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
    }
}

pub enum EmuMessage {   
    Start,
    Stop,
    HotSwap
}