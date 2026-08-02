use super::*;

#[inline]
pub(super) const fn code_mask<const BITS: u8>() -> u8 {
    match BITS {
        0 => 0,
        1..=7 => (1u8 << BITS) - 1,
        8 => u8::MAX,
        _ => panic!("packed character width must not exceed 8"),
    }
}

#[inline]
pub(super) fn checked_code<C, const BITS: u8>(character: C) -> u8
where
    C: PackedChar<BITS>,
{
    let code = character.code();
    assert_eq!(
        code & !code_mask::<BITS>(),
        0,
        "PackedChar::code does not fit in BITS"
    );
    assert!(
        C::from_code(code) == Some(character),
        "PackedChar::code and PackedChar::from_code disagree"
    );
    code
}

#[inline]
pub(super) fn write_code<const BITS: u8>(bits: &mut BitString, position: usize, code: u8) {
    let start = position * usize::from(BITS);
    bits.set_chunk(start, u64::from(code), usize::from(BITS));
}

#[cfg(test)]
mod tests_for_code;
