use std::{path::PathBuf, sync::mpsc::Sender};

use crate::{common::errors::HydraIOError, emulator::{EmuMessage, Emulator}, window::HydraApp};

pub struct N3ds;

impl Emulator for N3ds {
    const CONSOLE_NAME: &str = "3DS";
    const CORE_NAME: &str = "Lemonshark";
    const FILE_FILTERS: &[(&str, &[&str])] = &[super::N3DS_FILE_FILTER];

    type Model = ();
    
    fn main_thread(self) { todo!() }
    fn try_init(_model: Self::Model, _rom_path: &PathBuf, _app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> { todo!() }
}