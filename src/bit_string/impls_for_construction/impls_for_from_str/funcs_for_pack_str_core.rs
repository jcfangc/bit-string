use alloc::{boxed::Box, vec::Vec};

use crate::bit_string::bits::Bits;

/// Pack `bit_len` ASCII '0'/'1' bytes from `src` into a `Box<[u64]>`.
///
/// Returns `Err((index, byte))` on the first invalid character.
/// Bits are packed in little-endian order: byte `i` becomes bit `i % 64`
/// of word `i / 64`.
#[inline]
pub(super) fn str_core(src: *const u8, bit_len: usize) -> Result<Box<[u64]>, (usize, u8)> {
    let word_len = Bits::word_len(bit_len);
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of up to `word_len` u64 values.
    // - `dispatch` either writes all slots or returns an error (in which case
    //   the Vec is dropped without reading uninitialized memory).
    let error = unsafe { dispatch(out.as_mut_ptr(), src, bit_len) };

    if let Some((idx, byte)) = error {
        return Err((idx, byte));
    }

    // SAFETY: `dispatch` returned `None`, meaning it successfully wrote
    // every slot in `0..word_len`.
    unsafe { out.set_len(word_len) };

    Bits::mask_unused(&mut out, bit_len);
    Ok(out.into_boxed_slice())
}

/// Validates and packs `bit_len` ASCII '0'/'1' bytes.
///
/// Returns `None` on success, or `Some((index, invalid_byte))` on the first
/// byte that is neither b'0' (0x30) nor b'1' (0x31).
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u8, bit_len: usize) -> Option<(usize, u8)> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: forwarded from `dispatch`'s safety contract.
        return unsafe { avx2::words(dst, src, bit_len) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: forwarded from `dispatch`'s safety contract.
        return unsafe { sse2::words(dst, src, bit_len) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: forwarded from `dispatch`'s safety contract.
        return unsafe { neon::words(dst, src, bit_len) };
    }

    #[allow(unused)]
    // SAFETY: forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words(dst, src, bit_len)
    }
}

// ---------------------------------------------------------------------------
// Scalar backend
// ---------------------------------------------------------------------------

#[allow(unused)]
mod scalar {
    /// Validate-and-pack: process 64 bytes at a time.
    ///
    /// # Safety
    ///
    /// - `src` must be valid for reads of `bit_len` u8 values.
    /// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
    #[inline]
    pub(super) unsafe fn words(
        mut dst: *mut u64,
        mut src: *const u8,
        mut bit_len: usize,
    ) -> Option<(usize, u8)> {
        let mut global_offset = 0usize;

        while bit_len >= 64 {
            let mut word = 0u64;
            for i in 0..64 {
                // SAFETY: `i < 64` and `bit_len >= 64`.
                let byte = unsafe { src.add(i).read() };
                let bit = match byte {
                    b'0' => 0u64,
                    b'1' => 1u64,
                    _ => return Some((global_offset + i, byte)),
                };
                word |= bit << i;
            }

            // SAFETY: `dst` points to the next output slot.
            unsafe {
                *dst = word;
                dst = dst.add(1);
                src = src.add(64);
            }
            global_offset += 64;
            bit_len -= 64;
        }

        if bit_len > 0 {
            let mut word = 0u64;
            for i in 0..bit_len {
                // SAFETY: `i < bit_len`.
                let byte = unsafe { src.add(i).read() };
                let bit = match byte {
                    b'0' => 0u64,
                    b'1' => 1u64,
                    _ => return Some((global_offset + i, byte)),
                };
                word |= bit << i;
            }

            // SAFETY: `dst` points to the last output slot.
            unsafe {
                *dst = word;
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// AVX2 backend
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };

    const LANES: usize = 32;

    /// AVX2 backend: validate + pack, 32 bytes / lane.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words(
        mut dst: *mut u64,
        mut src: *const u8,
        mut bit_len: usize,
    ) -> Option<(usize, u8)> {
        let ones = _mm256_set1_epi8(b'1' as i8);
        let zeros = _mm256_set1_epi8(b'0' as i8);
        let mut global_offset = 0usize;

        while bit_len >= 64 {
            // Load 2 × 32 bytes.
            // SAFETY: `bit_len >= 64`, two unaligned 32-byte loads are in bounds.
            let lo = unsafe { _mm256_loadu_si256(src.cast::<__m256i>()) };
            let hi = unsafe { _mm256_loadu_si256(src.add(LANES).cast::<__m256i>()) };

            // Validate: each byte must equal b'0' or b'1'.
            // cmpeq(v, b'1') → 0xFF for '1', 0x00 otherwise.
            // cmpeq(v, b'0') → 0xFF for '0', 0x00 otherwise.
            let lo_ones = _mm256_cmpeq_epi8(lo, ones);
            let lo_zeros = _mm256_cmpeq_epi8(lo, zeros);
            let hi_ones = _mm256_cmpeq_epi8(hi, ones);
            let hi_zeros = _mm256_cmpeq_epi8(hi, zeros);

            let lo_valid = _mm256_or_si256(lo_ones, lo_zeros);
            let hi_valid = _mm256_or_si256(hi_ones, hi_zeros);

            let lo_valid_mask = _mm256_movemask_epi8(lo_valid) as u32;
            let hi_valid_mask = _mm256_movemask_epi8(hi_valid) as u32;

            if lo_valid_mask != 0xFFFF_FFFF || hi_valid_mask != 0xFFFF_FFFF {
                // Fall back to scalar within this 64-byte chunk for exact
                // error position.
                // SAFETY: src points at this chunk, dst points at current slot.
                let (i, b) =
                    unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
                return Some((global_offset + i, b));
            }

            // Pack: movemask extracts the MSB of cmpeq result.
            // b'1' → 0xFF (MSB=1), b'0' → 0x00 (MSB=0).
            let lo_bits = _mm256_movemask_epi8(lo_ones) as u32 as u64;
            let hi_bits = _mm256_movemask_epi8(hi_ones) as u32 as u64;

            // SAFETY: `dst` points to the next output slot.
            unsafe {
                *dst = lo_bits | (hi_bits << 32);
                dst = dst.add(1);
                src = src.add(64);
            }
            global_offset += 64;
            bit_len -= 64;
        }

        // Delegate tail to scalar.
        // SAFETY: `bit_len < 64`.
        unsafe {
            if let Some((i, b)) = scalar::words(dst, src, bit_len) {
                return Some((global_offset + i, b));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// SSE2 backend
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };

    const LANES: usize = 16;

    /// SSE2 backend: validate + pack, 16 bytes / lane.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn words(
        mut dst: *mut u64,
        mut src: *const u8,
        mut bit_len: usize,
    ) -> Option<(usize, u8)> {
        let ones = _mm_set1_epi8(b'1' as i8);
        let zeros = _mm_set1_epi8(b'0' as i8);
        let mut global_offset = 0usize;

        while bit_len >= 64 {
            // Load 4 × 16 bytes.
            // SAFETY: `bit_len >= 64`, four unaligned 16-byte loads are in bounds.
            let v0 = unsafe { _mm_loadu_si128(src.cast::<__m128i>()) };
            let v1 = unsafe { _mm_loadu_si128(src.add(LANES).cast::<__m128i>()) };
            let v2 = unsafe { _mm_loadu_si128(src.add(LANES * 2).cast::<__m128i>()) };
            let v3 = unsafe { _mm_loadu_si128(src.add(LANES * 3).cast::<__m128i>()) };

            let ones0 = _mm_cmpeq_epi8(v0, ones);
            let zeros0 = _mm_cmpeq_epi8(v0, zeros);
            let ones1 = _mm_cmpeq_epi8(v1, ones);
            let zeros1 = _mm_cmpeq_epi8(v1, zeros);
            let ones2 = _mm_cmpeq_epi8(v2, ones);
            let zeros2 = _mm_cmpeq_epi8(v2, zeros);
            let ones3 = _mm_cmpeq_epi8(v3, ones);
            let zeros3 = _mm_cmpeq_epi8(v3, zeros);

            let valid0 = _mm_or_si128(ones0, zeros0);
            let valid1 = _mm_or_si128(ones1, zeros1);
            let valid2 = _mm_or_si128(ones2, zeros2);
            let valid3 = _mm_or_si128(ones3, zeros3);

            let mask0 = _mm_movemask_epi8(valid0) as u16;
            let mask1 = _mm_movemask_epi8(valid1) as u16;
            let mask2 = _mm_movemask_epi8(valid2) as u16;
            let mask3 = _mm_movemask_epi8(valid3) as u16;

            if mask0 != 0xFFFF || mask1 != 0xFFFF || mask2 != 0xFFFF || mask3 != 0xFFFF {
                let (i, b) =
                    unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
                return Some((global_offset + i, b));
            }

            let b0 = _mm_movemask_epi8(ones0) as u32 as u64;
            let b1 = _mm_movemask_epi8(ones1) as u32 as u64;
            let b2 = _mm_movemask_epi8(ones2) as u32 as u64;
            let b3 = _mm_movemask_epi8(ones3) as u32 as u64;

            // SAFETY: `dst` points to the next output slot.
            unsafe {
                *dst = b0 | (b1 << 16) | (b2 << 32) | (b3 << 48);
                dst = dst.add(1);
                src = src.add(64);
            }
            global_offset += 64;
            bit_len -= 64;
        }

        // SAFETY: `bit_len < 64`.
        unsafe {
            if let Some((i, b)) = scalar::words(dst, src, bit_len) {
                return Some((global_offset + i, b));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// NEON backend (uses XOR approach: xor0 serves both validation and extraction)
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::scalar;

    use core::arch::aarch64::{
        vand_u8, vceq_u8, vdup_n_u8, veor_u8, vget_lane_u64, vld1_u8, vpaddl_u8, vpaddl_u16,
        vpaddl_u32, vreinterpret_u64_u8,
    };

    /// Bit-position masks for vpaddl reduction.
    const BIT_MASKS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

    /// NEON backend: validate + pack, 64 bytes at a time.
    ///
    /// Uses `(v ^ 0x30) & (v ^ 0x31)` for validation, reusing `v ^ 0x30`
    /// (which holds the bit value in LSB) for extraction.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn words(
        mut dst: *mut u64,
        mut src: *const u8,
        mut bit_len: usize,
    ) -> Option<(usize, u8)> {
        let zero_byte = vdup_n_u8(b'0');
        let one_byte = vdup_n_u8(b'1');
        let bit_masks = unsafe { vld1_u8(BIT_MASKS.as_ptr()) };
        let mut global_offset = 0usize;

        while bit_len >= 64 {
            // Validate all 8 groups, then pack them into one u64.
            let mut word = 0u64;
            for group in 0..8 {
                // SAFETY: `bit_len >= 64` and `group < 8`, so the
                // 8-byte read from `src + group*8` is in bounds.
                let v = unsafe { vld1_u8(src.add(group * 8)) };

                // `xor0 = v ^ 0x30`:
                //   b'0' → 0x00, b'1' → 0x01  (bit value in LSB)
                // `xor1 = v ^ 0x31`:
                //   b'0' → 0x01, b'1' → 0x00
                // `invalid = xor0 & xor1`:
                //   valid → 0x00, invalid → non-zero
                let xor0 = veor_u8(v, zero_byte);
                let xor1 = veor_u8(v, one_byte);
                let invalid = vand_u8(xor0, xor1);

                // Cheap all-zero test via reinterpret as u64.
                let invalid_u64 = vget_lane_u64::<0>(vreinterpret_u64_u8(invalid));
                if invalid_u64 != 0 {
                    // Fall back to scalar for exact error position within
                    // this 64-byte chunk.
                    let (i, b) =
                        unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
                    return Some((global_offset + i, b));
                }

                // Pack: xor0 holds the bit value (0x00 or 0x01).
                // Expand 0x01 → 0xFF via vceq_u8 so that vand with
                // bit-position masks correctly captures every lane,
                // then horizontal pairwise add collapses to one u64.
                let is_one = vceq_u8(xor0, vdup_n_u8(1));
                let masked = vand_u8(is_one, bit_masks);
                let sum16 = vpaddl_u8(masked);
                let sum32 = vpaddl_u16(sum16);
                let sum64 = vpaddl_u32(sum32);

                let group_bits = vget_lane_u64::<0>(sum64);
                word |= group_bits << (group * 8);
            }

            // SAFETY: `dst` points to the current output slot.
            unsafe {
                *dst = word;
                dst = dst.add(1);
                src = src.add(64);
            }
            global_offset += 64;
            bit_len -= 64;
        }

        // SAFETY: `bit_len < 64`.
        unsafe {
            if let Some((i, b)) = scalar::words(dst, src, bit_len) {
                return Some((global_offset + i, b));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
