use crate::{WORD_BITS, assert_valid_width};

#[inline]
pub(super) fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    assert_valid_width::<BITS>();
    let expected_codes = words.len() * WORD_BITS / usize::from(BITS);

    assert!(
        words.len() * WORD_BITS % usize::from(BITS) == 0,
        "word count must contain complete packed blocks"
    );
    assert_eq!(
        codes.len(),
        expected_codes,
        "output must contain exactly the codes represented by the input words"
    );

    dispatch::<BITS>(words, codes);
}

#[inline]
fn dispatch<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    if BITS == 4 && codes.len() >= 64 {
        let simd_code_len = codes.len() / 64 * 64;
        let simd_word_len = simd_code_len / 16;

        // SAFETY: The function is compiled only when AVX2 is enabled. The
        // prefix contains complete 64-code blocks and exactly four words each.
        unsafe { avx2::unpack_four_bit(&words[..simd_word_len], &mut codes[..simd_code_len]) };
        scalar::unpack_codes::<BITS>(&words[simd_word_len..], &mut codes[simd_code_len..]);
        return;
    }

    scalar::unpack_codes::<BITS>(words, codes);
}

mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
