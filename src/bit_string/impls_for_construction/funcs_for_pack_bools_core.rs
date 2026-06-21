use alloc::vec::Vec;

use crate::traits::*;
use crate::word_len;

/// Pack `bit_len` LSBs from `src` into a `Vec<u64>`.
///
/// Each source byte is treated as one bit (0 → 0, non-zero → 1).
/// Bits are packed in little-endian order: byte `i` becomes bit `i % 64`
/// of word `i / 64`.
#[inline]
pub(super) fn bools_core(src: *const u8, bit_len: usize) -> Vec<u64> {
    let word_len = word_len(bit_len);
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for exactly `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), src, bit_len);
        out.set_len(word_len);
    }

    out.mask_unused_bits(bit_len);
    out
}

/// Packs `bit_len` bytes from `src` into u64 words at `dst`.
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u8, bit_len: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is available.
        unsafe { avx2::words(dst, src, bit_len) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when SSE2 is available.
        unsafe { sse2::words(dst, src, bit_len) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is available.
        unsafe { neon::words(dst, src, bit_len) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words(dst, src, bit_len);
    }
}

#[allow(unused)]
mod scalar {
    use super::scalar;

    /// Scalar backend: accumulate 64 bytes at a time into one u64.
    ///
    /// # Safety
    ///
    /// - `src` must be valid for reads of `bit_len` u8 values.
    /// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
    #[inline]
    pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
        while bit_len >= 64 {
            // SAFETY: `bit_len >= 64`, so reading 64 bytes from `src` is valid.
            // `dst` points to the next output slot.
            unsafe {
                *dst = scalar::pack_64(src);
            }
            // SAFETY:
            // - `dst` advances by 1 word; the caller ensures the destination
            //   has enough capacity for all full-word writes.
            // - `src` advances by 64 bytes; `bit_len >= 64` ensures read bounds.
            unsafe {
                dst = dst.add(1);
                src = src.add(64);
            }
            bit_len -= 64;
        }

        if bit_len > 0 {
            // SAFETY: `bit_len > 0`, so reading `bit_len` bytes from `src` is valid.
            unsafe {
                *dst = scalar::pack_partial(src, bit_len);
            }
        }
    }

    /// Pack exactly 64 bytes into one u64 (little-endian: byte i → bit i).
    ///
    /// # Safety
    ///
    /// `src` must be valid for reads of 64 u8 values.
    #[inline]
    unsafe fn pack_64(src: *const u8) -> u64 {
        let mut word = 0u64;
        for i in 0..64 {
            // SAFETY:
            // - `i < 64`, offset is in bounds.
            // - `src` is valid for 64 reads per caller contract.
            let byte = unsafe { src.add(i).read() };
            word |= ((byte & 1) as u64) << i;
        }
        word
    }

    /// Pack fewer than 64 bytes into one u64.
    ///
    /// # Safety
    ///
    /// `src` must be valid for reads of `len` u8 values (`len < 64`).
    #[inline]
    unsafe fn pack_partial(src: *const u8, len: usize) -> u64 {
        let mut word = 0u64;
        for i in 0..len {
            // SAFETY:
            // - `i < len < 64`.
            // - `src` is valid for `len` reads per caller contract.
            let byte = unsafe { src.add(i).read() };
            word |= ((byte & 1) as u64) << i;
        }
        word
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    const LANES: usize = 32;

    /// AVX2 backend: 32 bytes → 32 movemask bits, 2 iterations per u64.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `src` must be valid for reads of `bit_len` u8 values.
    /// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
        // SAFETY: this function is only callable when AVX2 is available
        // (enforced by the caller / dispatch gating).
        let ones = _mm256_set1_epi8(1);

        while bit_len >= 64 {
            // SAFETY:
            // - `bit_len >= 64`, so two 32-byte reads from `src` are in bounds.
            // - `_mm256_loadu_si256` permits unaligned loads.
            let lo = unsafe { _mm256_loadu_si256(src.cast::<__m256i>()) };
            // SAFETY: `src + 32` is valid; `bit_len >= 64`.
            let hi = unsafe { _mm256_loadu_si256(src.add(LANES).cast::<__m256i>()) };

            // cmpeq extracts LSB: 0x01 → 0xFF, 0x00 → 0x00
            let lo_eq = _mm256_cmpeq_epi8(lo, ones);
            let hi_eq = _mm256_cmpeq_epi8(hi, ones);

            // movemask takes the MSB of each byte → the comparison result.
            let lo_bits = _mm256_movemask_epi8(lo_eq) as u32 as u64;
            let hi_bits = _mm256_movemask_epi8(hi_eq) as u32 as u64;

            // SAFETY: `dst` points to the next output slot.
            unsafe {
                *dst = lo_bits | (hi_bits << 32);
            }

            // SAFETY: destination has capacity; source has `bit_len >= 64`.
            unsafe {
                dst = dst.add(1);
                src = src.add(64);
            }
            bit_len -= 64;
        }

        // SAFETY: `bit_len < 64`, delegate tail to scalar.
        unsafe {
            scalar::words(dst, src, bit_len);
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    const LANES: usize = 16;

    /// SSE2 backend: 16 bytes → 16 movemask bits, 4 iterations per u64.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSE2 is available.
    /// - `src` must be valid for reads of `bit_len` u8 values.
    /// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
        let ones = _mm_set1_epi8(1);

        while bit_len >= 64 {
            // SAFETY: `bit_len >= 64`, so four 16-byte reads from `src` are in bounds.
            // `_mm_loadu_si128` permits unaligned loads.
            let v0 = unsafe { _mm_loadu_si128(src.cast::<__m128i>()) };
            let v1 = unsafe { _mm_loadu_si128(src.add(LANES).cast::<__m128i>()) };
            let v2 = unsafe { _mm_loadu_si128(src.add(LANES * 2).cast::<__m128i>()) };
            let v3 = unsafe { _mm_loadu_si128(src.add(LANES * 3).cast::<__m128i>()) };

            let m0 = _mm_movemask_epi8(_mm_cmpeq_epi8(v0, ones)) as u32 as u64;
            let m1 = _mm_movemask_epi8(_mm_cmpeq_epi8(v1, ones)) as u32 as u64;
            let m2 = _mm_movemask_epi8(_mm_cmpeq_epi8(v2, ones)) as u32 as u64;
            let m3 = _mm_movemask_epi8(_mm_cmpeq_epi8(v3, ones)) as u32 as u64;

            // SAFETY: `dst` points to the next output slot.
            unsafe {
                *dst = m0 | (m1 << 16) | (m2 << 32) | (m3 << 48);
            }

            // SAFETY: destination has capacity; source has `bit_len >= 64`.
            unsafe {
                dst = dst.add(1);
                src = src.add(64);
            }
            bit_len -= 64;
        }

        // SAFETY: `bit_len < 64`, delegate tail to scalar.
        unsafe {
            scalar::words(dst, src, bit_len);
        }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::scalar;

    use core::arch::aarch64::{
        vand_u8, vceq_u8, vdup_n_u8, vget_lane_u64, vld1_u8, vpaddl_u8, vpaddl_u16, vpaddl_u32,
    };

    /// Bit-position masks: [1, 2, 4, 8, 16, 32, 64, 128].
    const BIT_MASKS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

    /// NEON backend: packs 64 bytes at a time into one u64.
    ///
    /// Each 8-byte group contributes bits at `group * 8` offset within the
    /// current word, then 8 groups are OR'd together.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `src` must be valid for reads of `bit_len` u8 values.
    /// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
        // SAFETY: constant pointer to static mask array.
        let bit_masks = unsafe { vld1_u8(BIT_MASKS.as_ptr()) };
        let ones = vdup_n_u8(1);

        while bit_len >= 64 {
            // Accumulate 8 groups × 8 bytes → one u64.
            let mut word = 0u64;
            for group in 0..8 {
                // SAFETY: `bit_len >= 64` and `group < 8`, so the load
                // is within bounds. `vld1_u8` permits unaligned reads.
                let bytes = unsafe { vld1_u8(src.add(group * 8)) };

                // Expand 1 → 0xFF (0 stays 0x00), then mask to position
                // each bit before reducing to a single u64 via pairwise adds.
                let is_one = vceq_u8(bytes, ones);
                let masked = vand_u8(is_one, bit_masks);
                let sum16 = vpaddl_u8(masked);
                let sum32 = vpaddl_u16(sum16);
                let sum64 = vpaddl_u32(sum32);

                let group_bits = vget_lane_u64::<0>(sum64);
                word |= group_bits << (group * 8);
            }

            // SAFETY: `dst` points to the current output slot; destination
            // has capacity for this write per the caller's contract.
            unsafe {
                *dst = word;
                dst = dst.add(1);
                src = src.add(64);
            }
            bit_len -= 64;
        }

        // SAFETY: `bit_len < 64`, delegate tail to scalar.
        unsafe {
            scalar::words(dst, src, bit_len);
        }
    }
}

mod impls_for_from_bool_iter;
mod impls_for_from_bool_slice;

#[cfg(test)]
mod tests_for_backend_equivalence;
