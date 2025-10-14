use std::{ffi::OsStr, path::Path, sync::{Arc, RwLock}};

use crate::{common::errors::HydraIOError, config::Config, gameboy, graphics::Graphics};

pub trait Emulator {
    fn main_thread(self: Arc<Self>);
}

pub fn init_from_file(path: &Path, graphics: Arc<RwLock<Graphics>>, config: &Config) -> Result<Arc<dyn Emulator>, HydraIOError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("gb") => 
            gameboy::GameBoy::from_model(path, gameboy::Model::GameBoy(None), graphics, config).map(|emulator| Arc::new(emulator) as Arc<dyn Emulator>), //This is disgusting but I think it's still the least horrible
        Some("gbc") => 
            gameboy::GameBoy::from_model(path, gameboy::Model::GameBoyColor(None), graphics, config).map(|emulator| Arc::new(emulator) as Arc<dyn Emulator>),
        ext => 
            Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
    }
}
