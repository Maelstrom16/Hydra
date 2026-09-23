mod cpu;

use std::{path::{Path, PathBuf}, sync::mpsc::Sender};

use crate::{common::errors::HydraIOError, config, emulator::{EmuMessage, Emulator, gameboy::{self, Agb, GameBoy}}, window::HydraApp};

pub struct GameBoyAdvance;

impl Emulator for GameBoyAdvance {
    const CONSOLE_NAME: &str = "Game Boy Advance";
    const CORE_NAME: &str = "Lindwyrm";
    const FILE_FILTERS: &[(&str, &[&str])] = &[super::GBA_FILE_FILTER, super::GB_FILE_FILTER];

    type Model = GbaTarget;
    
    fn main_thread(self) { todo!() }
    fn try_init(model: Self::Model, rom_path: &PathBuf, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        match model {
            GbaTarget::Gb(_) => GameBoy::<Agb>::try_init(app.get_config().gb.default_models.agb, rom_path, app),
            GbaTarget::Gba => todo!()
        }
    }
}

pub enum GbaTarget {
    Gb(gameboy::AgbRevision),
    Gba
}