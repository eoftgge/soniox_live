pub fn is_silent(buffer: &[i16], threshold: u32) -> bool {
    if buffer.is_empty() { return true; }

    let mut sum_squares = 0u64;
    for &sample in buffer {
        let s = sample as i32;
        sum_squares += s.pow(2) as u64;
    }

    let limit = (threshold as u64).pow(2) * (buffer.len() as u64);
    sum_squares < limit
}