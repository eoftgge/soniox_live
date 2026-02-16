use cpal::DeviceId;
use cpal::traits::{DeviceTrait, HostTrait};

const SCALE: f32 = i16::MAX as f32;

pub fn convert_audio_chunk(input: &[f32], output: &mut Vec<i16>) {
    if !output.is_empty() {
        output.clear();
    }
    output.extend(input.iter().map(|&s| (s.clamp(-1.0, 1.0) * SCALE) as i16));
}

pub fn get_available_devices() -> Vec<DeviceId> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    let mut names = Vec::new();

    for device in devices {
        let id = device.id().unwrap();
        names.push(id);
    }

    names
}

pub fn get_default_device() -> DeviceId {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    device.id().unwrap()
}