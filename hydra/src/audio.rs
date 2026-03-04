use std::{collections::VecDeque, io::Read, sync::{Arc, Mutex}, time::Duration};

use cpal::{Device, Host, OutputCallbackInfo, SampleRate, SizedSample, Stream, StreamConfig, StreamError, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ringbuf::{HeapProd, HeapRb, traits::{Consumer, Observer, Split}, wrap::Wrap};

use crate::common::audio::sine_callback;

pub struct Audio {
    host: Host,
    output: Device,
    config: StreamConfig,
    stream: Option<Stream>,
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let output = host.default_output_device().unwrap();
        let supported_config = output.default_output_config().unwrap();
        let config = supported_config.config();
        Audio { host, output, config, stream: None }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn get_channel_count(&self) -> u16 {
        self.config.channels
    }

    pub fn get_producer(&mut self) -> HeapProd<f32> {
        let (producer, mut consumer) = HeapRb::<f32>::new(self.config.sample_rate as usize / 10).split();
        let stream = self.output.build_output_stream(&self.config, move |samples: &mut [f32], _| {consumer.pop_slice(samples);}, Self::error_callback, None).unwrap();
        stream.play();
        self.stream = Some(stream);
        return producer;
    }

    fn error_callback(err: StreamError) {
        panic!("Audio streaming error: {}", err)
    }
}