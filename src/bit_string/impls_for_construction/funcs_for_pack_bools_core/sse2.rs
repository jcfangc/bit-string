use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{__m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};

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
