use std::f32;

use cpal::{OutputCallbackInfo, Sample, StreamConfig, StreamError, traits::{DeviceTrait, HostTrait, StreamTrait}};

use crate::gameboy::memory::io::{IoMap, MMIO, deserialized::{RegNr10, RegNr11, RegNr12, RegNr13, RegNr14, RegNr21, RegNr22, RegNr23, RegNr24, RegNr30, RegNr31, RegNr32, RegNr33, RegNr34, RegNr41, RegNr42, RegNr43, RegNr44, RegNr50, RegNr51, RegNr52, RegWav00, RegWav01, RegWav02, RegWav03, RegWav04, RegWav05, RegWav06, RegWav07, RegWav08, RegWav09, RegWav10, RegWav11, RegWav12, RegWav13, RegWav14, RegWav15}};

pub struct Apu {
    nr10: RegNr10,
    nr11: RegNr11,
    nr12: RegNr12,
    nr13: RegNr13,
    nr14: RegNr14,

    nr21: RegNr21,
    nr22: RegNr22,
    nr23: RegNr23,
    nr24: RegNr24,

    nr30: RegNr30,
    nr31: RegNr31,
    nr32: RegNr32,
    nr33: RegNr33,
    nr34: RegNr34,

    nr41: RegNr41,
    nr42: RegNr42,
    nr43: RegNr43,
    nr44: RegNr44,

    nr50: RegNr50,
    nr51: RegNr51,
    nr52: RegNr52,

    wav00: RegWav00,
    wav01: RegWav01,
    wav02: RegWav02,
    wav03: RegWav03,
    wav04: RegWav04,
    wav05: RegWav05,
    wav06: RegWav06,
    wav07: RegWav07,
    wav08: RegWav08,
    wav09: RegWav09,
    wav10: RegWav10,
    wav11: RegWav11,
    wav12: RegWav12,
    wav13: RegWav13,
    wav14: RegWav14,
    wav15: RegWav15,
}

pub fn test() {
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    let supported_config = output.default_output_config().unwrap();
    let config = supported_config.into();
    let stream = output.build_output_stream(&config, sine_callback(880.0, &config), error_callback, None).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(3));
}

fn sine_callback(frequency: f32, config: &StreamConfig) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + use<> {
    let mut phase = 0f32;
    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as f32;

    let inc = ((frequency * f32::consts::TAU) / sample_rate) / channels;

    move |samples, _callback_info| {
        for sample in samples.iter_mut() {
            *sample = (phase.sin() + 1.0) / 2.0;
            phase += inc;
            phase %= f32::consts::TAU;
        }
    }
}

fn error_callback(err: StreamError) {
    panic!("Audio streaming error: {}", err)
}