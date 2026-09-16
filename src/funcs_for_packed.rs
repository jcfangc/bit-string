#[inline]
pub(crate) const fn assert_valid_width<const BITS: u8>() {
    assert!(
        BITS > 0 && BITS <= 8,
        "packed character width must be between 1 and 8"
    );
}

#[inline]
pub(crate) const fn code_mask<const BITS: u8>() -> u8 {
    assert_valid_width::<BITS>();
    match BITS {
        1..=7 => (1u8 << BITS) - 1,
        8 => u8::MAX,
        _ => unreachable!(),
    }
}

/// Extracts one fixed-width packed code from a word-aligned or offset view.
#[inline]
pub(crate) fn extract_code<const BITS: u8>(words: &[u64], bit_start: usize, index: usize) -> u8 {
    let bits = usize::from(BITS);
    let bit_index = bit_start + index * bits;
    let word_index = bit_index / 64;
    let offset = bit_index % 64;

    let mut code = words[word_index] >> offset;
    if 64 % bits != 0 && offset + bits > 64 {
        code |= words[word_index + 1] << (64 - offset);
    }

    (code & u64::from(code_mask::<BITS>())) as u8
}

#[cfg(test)]
mod tests_for_packed;
