const SCALE: f32 = i16::MAX as f32;

pub fn convert_audio_chunk(input: &[f32], output: &mut Vec<i16>, channels: u16, sample_rate: u32) {
    output.clear();

    let ch = channels as usize;
    let gain = 5.0;

    let ratio = sample_rate as f32 / 16000.0;

    let total_frames = input.len() / ch;
    let target_frames = (total_frames as f32 / ratio) as usize;

    for i in 0..target_frames {
        let src_frame_idx = (i as f32 * ratio) as usize;

        if src_frame_idx >= total_frames {
            break;
        }

        let frame_start = src_frame_idx * ch;
        let frame = &input[frame_start..frame_start + ch];

        let sum: f32 = frame.iter().sum();
        let mono_sample = (sum / ch as f32) * gain;

        output.push((mono_sample.clamp(-1.0, 1.0) * SCALE) as i16);
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
