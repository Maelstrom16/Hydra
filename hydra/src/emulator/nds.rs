mod cpu;

use std::{path::{Path, PathBuf}, sync::mpsc::Sender};

use crate::{common::errors::HydraIOError, emulator::{EmuMessage, Emulator}, window::HydraApp};

pub struct Nds;

impl Emulator for Nds {
    const CONSOLE_NAME: &str = "DS";
    const CORE_NAME: &str = "Amphisbaena";
    const FILE_FILTERS: &[(&str, &[&str])] = &[super::NDS_FILE_FILTER];

    type Model = ();
    
    fn main_thread(self) { todo!() }
    fn try_init(_model: Self::Model, _rom_path: &PathBuf, _app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> { todo!() }
}