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
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        if matches!(BITS, 1 | 3 | 4 | 5 | 6 | 7) && codes.len() >= 32 {
            // SAFETY: This branch is compiled only when AVX2 is enabled.
            unsafe { avx2::pack_codes::<BITS>(dst, codes) };
            return;
        }
    }

    scalar::pack_codes::<BITS>(dst, codes);
}

mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[cfg(test)]
mod tests_for_backend_equivalence;
