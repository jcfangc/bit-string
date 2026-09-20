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
        // Conservative thresholds selected from `benches/packed_thresholds.rs`.
        const AVX2_BITS_1_THRESHOLD: usize = 64;
        const AVX2_BITS_3_THRESHOLD: usize = 64;
        const AVX2_BITS_4_THRESHOLD: usize = 64;
        const AVX2_BITS_5_THRESHOLD: usize = 64;
        const AVX2_BITS_6_THRESHOLD: usize = 32;
        const AVX2_BITS_7_THRESHOLD: usize = 64;

        let use_avx2 = match BITS {
            1 => codes.len() >= AVX2_BITS_1_THRESHOLD,
            3 => codes.len() >= AVX2_BITS_3_THRESHOLD,
            4 => codes.len() >= AVX2_BITS_4_THRESHOLD,
            5 => codes.len() >= AVX2_BITS_5_THRESHOLD,
            6 => codes.len() >= AVX2_BITS_6_THRESHOLD,
            7 => codes.len() >= AVX2_BITS_7_THRESHOLD,
            _ => false,
        };

        if use_avx2 {
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
