use crate::{assert_valid_width, word_len};

#[inline]
pub(crate) const fn layout_block_len<const BITS: u8>() -> usize {
    64 / gcd(64, BITS as usize)
}

#[inline]
const fn gcd(mut lhs: usize, mut rhs: usize) -> usize {
    while rhs != 0 {
        let remainder = lhs % rhs;
        lhs = rhs;
        rhs = remainder;
    }
    lhs
}

#[inline]
pub(super) fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    assert_valid_width::<BITS>();
    let block_len = layout_block_len::<BITS>();
    let expected_words = word_len(codes.len() * usize::from(BITS));

    assert!(
        codes.len().is_multiple_of(block_len),
        "code count must contain complete packed blocks"
    );
    assert_eq!(
        dst.len(),
        expected_words,
        "destination must contain exactly the packed block words"
    );

    scalar::pack_codes::<BITS>(dst, codes);
}

mod scalar;

#[cfg(test)]
mod tests_for_backend_equivalence;
