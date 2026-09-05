use super::*;

#[inline]
pub(super) fn write_code<const BITS: u8>(bits: &mut BitString, position: usize, code: u8) {
    let start = position * usize::from(BITS);
    bits.set_chunk(start, u64::from(code), usize::from(BITS));
}
