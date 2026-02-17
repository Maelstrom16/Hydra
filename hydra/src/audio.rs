use std::time::Duration;

use cpal::{Device, Host, OutputCallbackInfo, SizedSample, Stream, StreamConfig, StreamError, traits::{DeviceTrait, HostTrait}};

pub struct Audio {
    host: Host,
    output: Device,
    config: StreamConfig
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let output = host.default_output_device().unwrap();
        let supported_config = output.default_output_config().unwrap();
        let config = supported_config.config();

        Audio { host, output, config }
    }

    pub fn build_output_stream<T, D, E>(&self, data_callback: D, error_callback: E, timeout: Option<Duration>) -> Stream where
        T: SizedSample,
        D: FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.output.build_output_stream(&self.config, data_callback, error_callback, timeout).unwrap()
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn get_channel_count(&self) -> u16 {
        self.config.channels
    }
}