const SCALE: f32 = i16::MAX as f32;

pub fn convert_audio_chunk(input: &[f32], output: &mut Vec<i16>) {
    if !output.is_empty() {
        output.clear();
    }
    output.extend(input.iter().map(|&s| (s.clamp(-1.0, 1.0) * SCALE) as i16));
}
