//! SIMD word-level equality for `starts_with`.
//!
//! At position 0, both haystack and prefix are word-aligned, so we can
//! compare LANES u64 words at once via `cmpeq` + `movemask`.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn starts_with_words(sw: &[u64], pw: &[u64], full_words: usize) -> bool {
    if full_words < SMALL_WORDS {
        for i in 0..full_words {
            if sw[i] != pw[i] {
                return false;
            }
        }
        return true;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return unsafe { avx2::eq_words(sw, pw, full_words) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse2::eq_words(sw, pw, full_words) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::eq_words(sw, pw, full_words) };
    }

    #[allow(unused)]
    {
        for i in 0..full_words {
            if sw[i] != pw[i] {
                return false;
            }
        }
        true
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 4 <= len {
            let a = unsafe { _mm256_loadu_si256(sw.as_ptr().add(i).cast::<__m256i>()) };
            let b = unsafe { _mm256_loadu_si256(pw.as_ptr().add(i).cast::<__m256i>()) };
            let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
            if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
                return false;
            }
            i += 4;
        }
        while i < len {
            if sw[i] != pw[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { _mm_loadu_si128(sw.as_ptr().add(i).cast::<__m128i>()) };
            let b = unsafe { _mm_loadu_si128(pw.as_ptr().add(i).cast::<__m128i>()) };
            let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
            if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
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
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { vld1q_u64(sw.as_ptr().add(i)) };
            let b = unsafe { vld1q_u64(pw.as_ptr().add(i)) };
            let cmp = unsafe { vceqq_u64(a, b) };
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
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
}
