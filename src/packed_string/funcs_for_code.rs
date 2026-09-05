use super::*;

#[inline]
pub(crate) const fn code_mask<const BITS: u8>() -> u8 {
    match BITS {
        1..=7 => (1u8 << BITS) - 1,
        8 => u8::MAX,
        _ => panic!("packed character width must be between 1 and 8"),
    }
}

#[inline]
pub(super) fn write_code<const BITS: u8>(bits: &mut BitString, position: usize, code: u8) {
    let start = position * usize::from(BITS);
    bits.set_chunk(start, u64::from(code), usize::from(BITS));
}
