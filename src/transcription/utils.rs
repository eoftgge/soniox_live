const SCALE: f32 = i16::MAX as f32;

pub fn convert_audio_chunk(
    input: &[f32],
    output: &mut Vec<i16>,
    channels: u16,
    sample_rate: u32
) {
    output.clear();

    let step = (sample_rate / 16000).max(1) as usize;

    if channels == 2 {
        for chunk in input.chunks_exact(2).step_by(step) {
            let mono_sample = (chunk[0] + chunk[1]) * 0.5;
            output.push((mono_sample.clamp(-1.0, 1.0) * SCALE) as i16);
        }
    } else {
        for chunk in input.chunks_exact(1).step_by(step) {
            output.push((chunk[0].clamp(-1.0, 1.0) * SCALE) as i16);
        }
    }
}

pub fn is_punctuation_or_symbol(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_punctuation() || "。，！？；：「」『』《》、…—।॥".contains(c))
}

pub fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x3040..=0x309F).contains(&u)
        || (0x30A0..=0x30FF).contains(&u)
        || (0xAC00..=0xD7AF).contains(&u)
}
