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
///
/// # Safety
///
/// `BITS` must be in `1..=8`, `bit_start` must be a multiple of `BITS`, and the
/// complete `BITS`-wide code at `bit_start + index * BITS` must lie within the
/// logical bit range represented by `words`. The arithmetic for that range
/// must fit in `usize`. These conditions ensure that the first backing word
/// exists and, when the code crosses a word boundary, that the following
/// backing word exists as well.
#[inline]
pub(crate) unsafe fn extract_code_unchecked<const BITS: u8>(
    words: &[u64],
    bit_start: usize,
    index: usize,
) -> u8 {
    let bits = usize::from(BITS);
    let bit_index = bit_start + index * bits;
    let word_index = bit_index / 64;
    let offset = bit_index % 64;

    // SAFETY: The caller guarantees that the complete code lies in the logical
    // bit range, so its first backing word is present.
    let mut code = unsafe { *words.get_unchecked(word_index) } >> offset;
    if 64 % bits != 0 && offset + bits > 64 {
        // SAFETY: This branch is taken only when the code crosses the current
        // word boundary. Since the complete code lies in the logical bit
        // range, the following backing word is present.
        code |= unsafe { *words.get_unchecked(word_index + 1) } << (64 - offset);
    }

    (code & u64::from(code_mask::<BITS>())) as u8
}

#[cfg(test)]
mod tests_for_packed;
