const SCALE: f32 = i16::MAX as f32;

pub fn convert_audio_chunk(input: &[f32], output: &mut Vec<i16>) {
    if !output.is_empty() {
        output.clear();
    }
    output.extend(input.iter().map(|&s| (s.clamp(-1.0, 1.0) * SCALE) as i16));
}

pub fn is_silent(buffer: &[i16]) -> bool {
    let threshold = 100.0;

    if buffer.is_empty() { return true; }

    let mut sum_squares = 0.0;
    for &sample in buffer {
        let s = sample as f32;
        sum_squares += s.powi(2);
    }

    let rms = (sum_squares / buffer.len() as f32).sqrt();
    rms < threshold
}

pub fn is_punctuation_or_symbol(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() { return false; }
    s.chars().all(|c| c.is_ascii_punctuation() || "。，！？；：「」『』《》、…—।॥".contains(c))
}

pub fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u) ||
        (0x3400..=0x4DBF).contains(&u) ||
        (0x3040..=0x309F).contains(&u) ||
        (0x30A0..=0x30FF).contains(&u) ||
        (0xAC00..=0xD7AF).contains(&u)
}