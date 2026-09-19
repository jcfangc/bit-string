use crate::{assert_valid_width, code_mask, word_len};

#[inline]
pub(super) fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    assert_valid_width::<BITS>();
    let layout_len = super::layout_block_len::<BITS>();
    let expected_words = word_len(codes.len() * usize::from(BITS));

    assert!(
        codes.len().is_multiple_of(layout_len),
        "code count must contain complete packed blocks"
    );
    assert_eq!(
        dst.len(),
        expected_words,
        "destination must contain exactly the packed block words"
    );

    debug_assert!(
        codes.iter().all(|&code| code <= code_mask::<BITS>()),
        "packed code does not fit in BITS bits"
    );

    dispatch::<BITS>(dst, codes);
}

#[inline]
fn dispatch<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    scalar::pack_codes::<BITS>(dst, codes);
}

mod scalar;

#[cfg(test)]
mod tests_for_backend_equivalence;
