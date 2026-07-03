//! SIMD shifted-window copy.
//!
//! Copies `count` shifted 64-bit windows from `src` to `dst`:
//!
//! ```text
//! dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift))
//! ```
//!
//! The destination is always word-aligned.  Short inputs fall back to scalar.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Copy `count` shifted 64-bit windows from `src` into `dst`.
///
/// The caller guarantees `dst` has room for `count` words, `src` has at
/// least `count + 1` words, and `shift ∈ [1, WORD_BITS)`.
#[inline]
pub(super) fn copy_words_shifted(dst: &mut [u64], src: &[u64], count: usize, shift: usize) {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        for i in 0..count {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        }
        return;
    }

    // ── Default: runtime SIMD detection ─────────────────────────
    #[cfg(not(feature = "compile-time-dispatch"))]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let (has_avx2, has_sse2) = {
                #[cfg(target_arch = "x86_64")]
                {
                    // SAFETY: `__cpuid_count` is always safe to call on x86/x86_64 — it is a read-only instruction that queries CPU capabilities.
                    let leaf1 = unsafe { core::arch::x86_64::__cpuid_count(1, 0) };
                    // SAFETY: `__cpuid_count` is always safe to call on x86/x86_64 — it is a read-only instruction that queries CPU capabilities.
                    let leaf7 = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
                    (leaf7.ebx & (1 << 5) != 0, leaf1.edx & (1 << 26) != 0)
                }
                #[cfg(target_arch = "x86")]
                {
                    // SAFETY: `__cpuid_count` is always safe to call on x86/x86_64 — it is a read-only instruction that queries CPU capabilities.
                    let leaf1 = unsafe { core::arch::x86::__cpuid_count(1, 0) };
                    // SAFETY: `__cpuid_count` is always safe to call on x86/x86_64 — it is a read-only instruction that queries CPU capabilities.
                    let leaf7 = unsafe { core::arch::x86::__cpuid_count(7, 0) };
                    (leaf7.ebx & (1 << 5) != 0, leaf1.edx & (1 << 26) != 0)
                }
            };
            if has_avx2 {
                // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). AVX2 availability was confirmed by CPUID.
                unsafe { avx2::copy_words_shifted(dst, src, count, shift) };
                return;
            }
            if has_sse2 {
                // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). SSE2 availability was confirmed by CPUID.
                unsafe { sse2::copy_words_shifted(dst, src, count, shift) };
                return;
            }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). NEON availability is confirmed at compile time by `target_feature = "neon"`.
            unsafe { neon::copy_words_shifted(dst, src, count, shift) };
            return;
        }
        #[allow(unused)]
        {
            for i in 0..count {
                dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            }
        }
    }

    // ── compile-time-dispatch: pure #[cfg] cascade ──────────────
    #[cfg(feature = "compile-time-dispatch")]
    {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). AVX2 availability is confirmed at compile time.
            unsafe { avx2::copy_words_shifted(dst, src, count, shift) };
            return;
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). SSE2 availability is confirmed at compile time.
            unsafe { sse2::copy_words_shifted(dst, src, count, shift) };
            return;
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). NEON availability is confirmed at compile time.
            unsafe { neon::copy_words_shifted(dst, src, count, shift) };
            return;
        }

        #[allow(unused)]
        {
            for i in 0..count {
                dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AVX2 — 4 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        // SAFETY: `#[target_feature(enable = "avx2")]` guarantees the CPU supports AVX2; the `unsafe fn` contract guarantees pointer validity.
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        // SAFETY: Same as above.
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 4 <= len {
            // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee); `_mm256_loadu_si256` uses unaligned loads so alignment is not required.
            let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            // SAFETY: Same as above.
            let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
            // SAFETY: Pure register operations; no memory access.
            let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
            // SAFETY: Same as above.
            let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
            // SAFETY: Same as above.
            let window = unsafe { _mm256_or_si256(lo, hi) };
            // SAFETY: `dst` pointer is valid for `len` words (caller guarantee); `_mm256_storeu_si256` uses unaligned stores.
            unsafe { _mm256_storeu_si256(dst.as_mut_ptr().add(i).cast::<__m256i>(), window) };
            i += 4;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// SSE2 — 2 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        // SAFETY: `#[target_feature(enable = "sse2")]` guarantees the CPU supports SSE2; the `unsafe fn` contract guarantees pointer validity.
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        // SAFETY: Same as above.
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee); `_mm_loadu_si128` uses unaligned loads so alignment is not required.
            let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            // SAFETY: Same as above.
            let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
            // SAFETY: Pure register operations; no memory access.
            let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
            // SAFETY: Same as above.
            let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
            // SAFETY: Same as above.
            let window = unsafe { _mm_or_si128(lo, hi) };
            // SAFETY: `dst` pointer is valid for `len` words (caller guarantee); `_mm_storeu_si128` uses unaligned stores.
            unsafe { _mm_storeu_si128(dst.as_mut_ptr().add(i).cast::<__m128i>(), window) };
            i += 2;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use crate::WORD_BITS;
    use core::arch::aarch64::{vdupq_n_s64, vld1q_u64, vorrq_u64, vshlq_u64, vst1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        // SAFETY: `#[target_feature(enable = "neon")]` guarantees the CPU supports NEON; the `unsafe fn` contract guarantees pointer validity.
        let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
        // SAFETY: Same as above.
        let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee).
            let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            // SAFETY: Same as above.
            let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };
            // SAFETY: Pure register operations; no memory access.
            let lo = unsafe { vshlq_u64(w0, neg_shift) };
            // SAFETY: Same as above.
            let hi = unsafe { vshlq_u64(w1, pos_shift) };
            // SAFETY: Same as above.
            let window = unsafe { vorrq_u64(lo, hi) };
            // SAFETY: `dst` pointer is valid for `len` words (caller guarantee).
            unsafe { vst1q_u64(dst.as_mut_ptr().add(i), window) };
            i += 2;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
