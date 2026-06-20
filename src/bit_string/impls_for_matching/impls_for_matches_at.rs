use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

use super::*;

impl BitString {
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        if index > self.bit_len {
            return false;
        }

        if pattern.bit_len > self.bit_len - index {
            return false;
        }

        bits_equal_at(self, index, pattern)
    }

    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        if prefix.bit_len > self.bit_len {
            return false;
        }

        let pw = prefix.as_words();
        let sw: &[u64] = &self.words;
        let full_words = prefix.bit_len / WORD_BITS;

        // Word-aligned at position 0 — can use SIMD equality on chunks.
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            if !unsafe { starts_with_avx2(sw, pw, full_words) } {
                return false;
            }
        }
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            if !unsafe { starts_with_sse2(sw, pw, full_words) } {
                return false;
            }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if !unsafe { starts_with_neon(sw, pw, full_words) } {
                return false;
            }
        }
        #[cfg(not(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                any(target_feature = "avx2", target_feature = "sse2")
            ),
            all(target_arch = "aarch64", target_feature = "neon"),
        )))]
        {
            for i in 0..full_words {
                if sw[i] != pw[i] {
                    return false;
                }
            }
        }

        let rem = prefix.bit_len % WORD_BITS;
        if rem > 0 {
            let mask = low_mask(rem);
            if (sw[full_words] & mask) != (pw[full_words] & mask) {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        suffix.bit_len <= self.bit_len && self.matches_at(self.bit_len - suffix.bit_len, suffix)
    }
}

// -----------------------------------------------------------------------
// SIMD helpers for starts_with word comparison
// -----------------------------------------------------------------------

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[target_feature(enable = "avx2")]
unsafe fn starts_with_avx2(sw: &[u64], pw: &[u64], len: usize) -> bool {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

    let mut i = 0;
    while i + 4 <= len {
        let a = _mm256_loadu_si256(sw.as_ptr().add(i).cast::<__m256i>());
        let b = _mm256_loadu_si256(pw.as_ptr().add(i).cast::<__m256i>());
        let cmp = _mm256_cmpeq_epi64(a, b);
        if _mm256_movemask_pd(core::mem::transmute(cmp)) as u32 != 0b1111 {
            return false;
        }
        i += 4;
    }
    // scalar tail
    while i < len {
        if sw[i] != pw[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
#[target_feature(enable = "sse2")]
unsafe fn starts_with_sse2(sw: &[u64], pw: &[u64], len: usize) -> bool {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

    let mut i = 0;
    while i + 2 <= len {
        let a = _mm_loadu_si128(sw.as_ptr().add(i).cast::<__m128i>());
        let b = _mm_loadu_si128(pw.as_ptr().add(i).cast::<__m128i>());
        let cmp = _mm_cmpeq_epi64(a, b);
        if _mm_movemask_epi8(cmp) as u32 != 0xFFFF {
            return false;
        }
        i += 2;
    }
    while i < len {
        if sw[i] != pw[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
unsafe fn starts_with_neon(sw: &[u64], pw: &[u64], len: usize) -> bool {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

    let mut i = 0;
    while i + 2 <= len {
        let a: uint64x2_t = vld1q_u64(sw.as_ptr().add(i));
        let b: uint64x2_t = vld1q_u64(pw.as_ptr().add(i));
        let cmp = vceqq_u64(a, b);
        if vgetq_lane_u64(cmp, 0) == 0 || vgetq_lane_u64(cmp, 1) == 0 {
            return false;
        }
        i += 2;
    }
    while i < len {
        if sw[i] != pw[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests_for_matches_at;

#[cfg(test)]
mod tests_for_starts_with;

#[cfg(test)]
mod tests_for_ends_with;
