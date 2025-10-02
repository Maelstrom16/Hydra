use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    window_size: (u32, u32),
    gb: GBConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GBConfig {
    default_models: GBDefaultModelsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GBDefaultModelsConfig {
    dmg: crate::gameboy::Model,
    sgb: crate::gameboy::Model,
    cgb: crate::gameboy::Model,
    agb: crate::gameboy::Model,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            window_size: (800, 600),
            gb: GBConfig {
                default_models: GBDefaultModelsConfig {
                    dmg: crate::gameboy::Model::MGB,
                    sgb: crate::gameboy::Model::SGB2,
                    cgb: crate::gameboy::Model::CGB,
                    agb: crate::gameboy::Model::AGB,
                },
            },
        }
    }
}
