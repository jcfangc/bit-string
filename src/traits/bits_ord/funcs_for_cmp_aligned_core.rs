use core::cmp::Ordering;

use crate::SMALL_WORDS;
use crate::traits::BitOrd;

/// Returns `Some(Ordering)` if the first `count` aligned words of `src`
/// and `other` differ, otherwise `None` (all equal).
///
/// Dispatches to the best available SIMD backend at compile time.
/// Short inputs fall back to scalar.
#[inline]
pub(super) fn cmp_aligned_words(src: &[u64], other: &[u64], count: usize) -> Option<Ordering> {
    if count < SMALL_WORDS {
        return scalar_cmp_aligned(src, other, count);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return unsafe { avx2::cmp_aligned(src, other, count) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse2::cmp_aligned(src, other, count) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::cmp_aligned(src, other, count) };
    }

    #[allow(unreachable_code)]
    scalar_cmp_aligned(src, other, count)
}

#[inline]
fn scalar_cmp_aligned(src: &[u64], other: &[u64], count: usize) -> Option<Ordering> {
    for i in 0..count {
        if src[i] != other[i] {
            return Some(BitOrd::bitwise_cmp(src[i], other[i]));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// AVX2 — 4 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use core::cmp::Ordering;

    use crate::traits::BitOrd;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
        let mut i = 0;
        while i + 4 <= len {
            let a = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
            let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
            let mask = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;
            if mask != 0b1111 {
                let lane = mask.trailing_ones() as usize;
                return Some(BitOrd::bitwise_cmp(src[i + lane], other[i + lane]));
            }
            i += 4;
        }
        while i < len {
            if src[i] != other[i] {
                return Some(BitOrd::bitwise_cmp(src[i], other[i]));
            }
            i += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// SSE2 — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use core::cmp::Ordering;

    use crate::traits::BitOrd;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
            let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
            let mask = unsafe { _mm_movemask_epi8(cmp) } as u32;
            if mask != 0xFFFF {
                // Each lane is 8 bytes → 8 high bits.  The first zero byte
                // tells us which lane differs.
                let lane = mask.trailing_ones() as usize / 8;
                return Some(BitOrd::bitwise_cmp(src[i + lane], other[i + lane]));
            }
            i += 2;
        }
        while i < len {
            if src[i] != other[i] {
                return Some(BitOrd::bitwise_cmp(src[i], other[i]));
            }
            i += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::cmp::Ordering;

    use crate::traits::BitOrd;
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            let b = unsafe { vld1q_u64(other.as_ptr().add(i)) };
            let cmp = unsafe { vceqq_u64(a, b) };
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 {
                return Some(BitOrd::bitwise_cmp(src[i], other[i]));
            }
            if unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                return Some(BitOrd::bitwise_cmp(src[i + 1], other[i + 1]));
            }
            i += 2;
        }
        while i < len {
            if src[i] != other[i] {
                return Some(BitOrd::bitwise_cmp(src[i], other[i]));
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
