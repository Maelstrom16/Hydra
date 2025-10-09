use std::{ffi::OsStr, path::Path};

use crate::{common::errors::HydraIOError, config::Config, gameboy};

pub trait Emulator {
    fn main_thread(&mut self);
}

pub fn init_from_file(path: &Path, config: &Config) -> Result<Box<dyn Emulator>, HydraIOError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("gb") => gameboy::GameBoy::from_model(path, gameboy::Model::GameBoy(None), config).map(|emulator| Box::new(emulator) as Box<dyn Emulator>), //TODO This is disgusting but I think it's still the least horrible
        Some("gbc") => gameboy::GameBoy::from_model(path, gameboy::Model::GameBoyColor(None), config).map(|emulator| Box::new(emulator) as Box<dyn Emulator>),
        ext => Err(HydraIOError::InvalidEmulator("Hydra", ext.map(str::to_string))),
    }
}
