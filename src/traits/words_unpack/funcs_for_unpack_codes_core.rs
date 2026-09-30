use crate::{WORD_BITS, assert_valid_width, traits::layout_block_len};

#[inline]
pub(super) fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    assert_valid_width::<BITS>();
    let expected_words = codes.len() * usize::from(BITS) / WORD_BITS;

    assert!(
        codes.len().is_multiple_of(layout_block_len::<BITS>()),
        "code count must contain complete packed blocks"
    );
    assert_eq!(
        words.len(),
        expected_words,
        "input must contain exactly the words represented by the codes"
    );

    dispatch::<BITS>(words, codes);
}

#[inline]
fn dispatch<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        const AVX2_BITS_4_THRESHOLD: usize = 64;

        if BITS == 4 && codes.len() >= AVX2_BITS_4_THRESHOLD {
            // SAFETY: This branch is compiled only when AVX2 is enabled, and
            // the core has validated the complete-block input/output contract.
            unsafe { avx2::unpack_codes::<BITS>(words, codes) };
            return;
        }
    }

    scalar::unpack_codes::<BITS>(words, codes);
}

mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[cfg(test)]
mod tests_for_backend_equivalence;
