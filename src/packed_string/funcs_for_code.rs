use super::*;

#[inline]
pub(super) fn assert_valid_width<C: PackedChar>() {
    assert!(C::BITS <= 8, "PackedChar::BITS must not exceed 8");
}

#[inline]
pub(super) fn code_mask<C: PackedChar>() -> u8 {
    match C::BITS {
        0 => 0,
        1..=7 => (1u8 << C::BITS) - 1,
        8 => u8::MAX,
        _ => unreachable!("PackedChar::BITS must not exceed 8"),
    }
}

#[inline]
pub(super) fn checked_code<C: PackedChar>(character: C) -> u8 {
    assert_valid_width::<C>();
    let code = character.code();
    assert_eq!(
        code & !code_mask::<C>(),
        0,
        "PackedChar::code does not fit in PackedChar::BITS"
    );
    assert!(
        C::from_code(code) == Some(character),
        "PackedChar::code and PackedChar::from_code disagree"
    );
    code
}

#[inline]
pub(super) fn write_code(bits: &mut BitString, position: usize, width: u8, code: u8) {
    let start = position * usize::from(width);
    bits.set_chunk(start, u64::from(code), usize::from(width));
}

#[cfg(test)]
mod tests_for_code;
