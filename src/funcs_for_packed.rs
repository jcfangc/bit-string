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

#[cfg(test)]
mod tests_for_packed;
