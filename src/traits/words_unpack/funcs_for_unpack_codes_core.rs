use crate::{WORD_BITS, assert_valid_width, traits::layout_block_len};

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const AVX2_BITS_1_THRESHOLD: usize = 64;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const AVX2_BITS_2_THRESHOLD: usize = 64;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const AVX2_BITS_4_THRESHOLD: usize = 64;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const AVX2_BITS_6_THRESHOLD: usize = 32;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const AVX2_BITS_8_THRESHOLD: usize = 64;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const fn avx2_threshold<const BITS: u8>() -> Option<usize> {
    match BITS {
        1 => Some(AVX2_BITS_1_THRESHOLD),
        2 => Some(AVX2_BITS_2_THRESHOLD),
        4 => Some(AVX2_BITS_4_THRESHOLD),
        6 => Some(AVX2_BITS_6_THRESHOLD),
        8 => Some(AVX2_BITS_8_THRESHOLD),
        _ => None,
    }
}

pub(crate) const fn has_accelerated_bulk_unpack<const BITS: u8>() -> bool {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return avx2_threshold::<BITS>().is_some();
    }

    #[cfg(not(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    )))]
    {
        false
    }
}

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
        // Thresholds match complete AVX2 kernel blocks. BITS=6 uses its
        // 32-code block; the other retained kernels process 64 codes.
        if avx2_threshold::<BITS>().is_some_and(|threshold| codes.len() >= threshold) {
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
