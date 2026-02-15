mod deserializers;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}};

use crate::common::audio;

pub struct Apu {            
    div: u8,
}

impl Apu {
    pub fn new() -> Self {
        Apu { div: 0 } // TODO: stub
    }

    pub fn tick(&mut self) {
        self.div = self.div.wrapping_add(1);
    }
}

pub fn test() {
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    let supported_config = output.default_output_config().unwrap();
    let config = supported_config.config();
    let stream = output.build_output_stream(&config, audio::sawtooth_callback(440.0, &config), audio::error_callback, None).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(3));
}