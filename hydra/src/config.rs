use std::fs;

use serde::{Deserialize, Serialize};

use crate::{common::errors::HydraIOError, propagate, propagate_or};

const CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub window_size: (u32, u32),
    pub gb: GBConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GBConfig {
    pub default_models: GBDefaultModelsConfig,
    pub show_all_revisions: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GBDefaultModelsConfig {
    pub dmg: crate::emulator::gameboy::DmgRevision,
    pub sgb: crate::emulator::gameboy::SgbRevision,
    pub cgb: crate::emulator::gameboy::CgbRevision,
    pub agb: crate::emulator::gameboy::AgbRevision,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            window_size: (800, 600),
            gb: GBConfig {
                default_models: GBDefaultModelsConfig {
                    dmg: crate::emulator::gameboy::DmgRevision::MGB,
                    sgb: crate::emulator::gameboy::SgbRevision::SGB2,
                    cgb: crate::emulator::gameboy::CgbRevision::CGB,
                    agb: crate::emulator::gameboy::AgbRevision::AGB,
                },
                show_all_revisions: false,
            },
        }
    }
}

impl Config {
    pub fn from_toml() -> Self {
        let args: Vec<String> = std::env::args().collect();
        if args.contains(&String::from("-i")) {
            // If initialize flag is set, delete config file (to be reset below).
            println!("Resetting config.toml.");
            if let Err(e) = std::fs::remove_file(CONFIG_PATH) {
                println!("Failed to delete config.toml: {}\nProgram will continue using old configurations.", e);
            }
        }
        let config = propagate_or!(toml::from_slice::<Config>(std::fs::read(CONFIG_PATH)?.as_slice())?, Config::default());
        return config;
    }

    pub fn write_to_toml(&self) {
        if let Err(e) = propagate!(fs::write(CONFIG_PATH, toml::to_string_pretty(self)?)?) {
            println!("Failed to save config.toml: {}\nProgram will continue using old configurations.", e);
        }
    }
}
