#[inline]
pub(super) fn matching_lanes(
    windows: &[u64; 4],
    needle_first_word: u64,
    needle_first_word_mask: u64,
    candidate_count: usize,
) -> u8 {
    if candidate_count == 4 {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            // SAFETY: The AVX2 backend is compiled only when AVX2 is enabled
            // for this target. `windows` contains exactly four readable lanes.
            return unsafe {
                avx2::matching_lanes(windows, needle_first_word, needle_first_word_mask)
            };
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1",
            not(target_feature = "avx2")
        ))]
        {
            // SAFETY: The SSE4.1 backend is compiled only when SSE4.1 is
            // enabled for this target. `windows` contains four readable lanes.
            return unsafe {
                sse41::matching_lanes(windows, needle_first_word, needle_first_word_mask)
            };
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: The NEON backend is compiled only when NEON is enabled
            // for this target. `windows` contains four readable lanes.
            return unsafe {
                neon::matching_lanes(windows, needle_first_word, needle_first_word_mask)
            };
        }
    }

    scalar::matching_lanes(
        windows,
        needle_first_word,
        needle_first_word_mask,
        candidate_count,
    )
}

#[inline]
pub(super) fn matching_candidates<const BITS: u8>(
    _haystack: crate::BitStr<'_>,
    _first_char: usize,
    _needle_first_word: u64,
    _needle_first_word_mask: u64,
) -> Option<u8> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: This module is compiled for AVX2 targets, and the backend
        // checks the backing slice bounds before issuing its gathers.
        return unsafe {
            avx2::matching_candidates::<BITS>(
                _haystack,
                _first_char,
                _needle_first_word,
                _needle_first_word_mask,
            )
        };
    }

    #[allow(unreachable_code)]
    None
}

mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
