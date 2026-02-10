use std::f32;

use cpal::{OutputCallbackInfo, StreamConfig, StreamError};

pub fn sine_callback(frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<> {
    let sample_rate = config.sample_rate as f32; // samples per second
    let channels = config.channels as usize; // 1 for mono, 2 for stereo
    let inc = (frequency * f32::consts::TAU) / sample_rate;
    let mut phase = 0f32;
    move |samples, _callback_info| {
        for sample in samples.chunks_mut(channels) {
            sample.fill(phase.sin());
            phase = (phase + inc) % f32::consts::TAU;
        }
    }
}

pub fn pulse_callback(duty: f32, frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<> {
    let sample_rate = config.sample_rate as f32; // samples per second
    let channels = config.channels as usize; // 1 for mono, 2 for stereo
    let inc = frequency / sample_rate;
    let mut phase = 0f32;
    move |samples, _callback_info| {
        for sample in samples.chunks_mut(channels) {
            sample.fill((duty - phase).signum());
            phase = (phase + inc) % 1.0;
        }
    }
}

pub fn triangle_callback(frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<> {
    let sample_rate = config.sample_rate as f32; // samples per second
    let channels = config.channels as usize; // 1 for mono, 2 for stereo
    let inc = (frequency * f32::consts::TAU) / sample_rate;
    let mut phase = 0f32;
    move |samples, _callback_info| {
        for sample in samples.chunks_mut(channels) {
            sample.fill(f32::consts::FRAC_2_PI * phase.sin().asin());
            phase = (phase + inc) % f32::consts::TAU;
        }
    }
}

pub fn sawtooth_callback(frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<> {
    let sample_rate = config.sample_rate as f32; // samples per second
    let channels = config.channels as usize; // 1 for mono, 2 for stereo
    let inc = (2.0 * frequency) / sample_rate;
    let mut phase = 1f32;
    move |samples, _callback_info| {
        for sample in samples.chunks_mut(channels) {
            sample.fill(phase - 1.0);
            phase = (phase + inc) % 2.0;
        }
    }
}

pub fn wave_callback<const N: usize>(wave: [f32; N], frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<N> {
    let sample_rate = config.sample_rate as f32; // samples per second
    let channels = config.channels as usize; // 1 for mono, 2 for stereo
    let inc = (N as f32 * frequency) / sample_rate;
    let mut phase = 0f32;
    move |samples, _callback_info| {
        for sample in samples.chunks_mut(channels) {
            sample.fill(wave[phase.floor() as usize]);
            phase = (phase + inc) % N as f32;
        }
    }
}

pub fn error_callback(err: StreamError) {
    panic!("Audio streaming error: {}", err)
}