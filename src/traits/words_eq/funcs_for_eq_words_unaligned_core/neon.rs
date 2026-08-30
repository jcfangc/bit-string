use crate::WORD_BITS;
use core::arch::aarch64::{
    vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
};

#[target_feature(enable = "neon")]
pub(super) unsafe fn eq_words_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> bool {
    // SAFETY: `shift` is in [1, WORD_BITS); the caller guarantees this
    // via the `shift == 0` fast-path in the entry point.
    // Both shift vectors fit in i64:
    //   shift ∈ [1, 63]  →  -shift ∈ [-63, -1]
    //   WORD_BITS - shift ∈ [1, 63]
    let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
    // SAFETY: same as above
    let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

    // Process 2 lanes (128 bits) per iteration.
    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `#[target_feature(enable = "neon")]` ensures NEON is enabled.
        // Pointers `src` and `other` are valid for `len+1` and `len` elements respectively (caller guarantees extra word for window shift).
        // Load [src[i], src[i+1]] and [src[i+1], src[i+2]].
        let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
        // SAFETY: same as above; load from `src[i+1]`.
        let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };

        // Build the shifted 64-bit window for each lane:
        //   window[k] = (src[i+k] >> shift) | (src[i+k+1] << (64 - shift))
        // vshlq_u64 with a negative shift amount performs a logical right shift.
        // SAFETY: `vshlq_u64`, `vorrq_u64`, `vceqq_u64`, and `vgetq_lane_u64` are pure register operations; NEON is enabled by `#[target_feature]`.
        let lo = unsafe { vshlq_u64(w0, neg_shift) };
        // SAFETY: same as above
        let hi = unsafe { vshlq_u64(w1, pos_shift) };
        // SAFETY: same as above
        let window = unsafe { vorrq_u64(lo, hi) };

        // SAFETY: NEON is enabled; pointer `other` is valid for `len` elements.
        let expected = unsafe { vld1q_u64(other.as_ptr().add(i)) };
        // SAFETY: `vceqq_u64` is a pure register operation; NEON is enabled by `#[target_feature]`.
        let cmp = unsafe { vceqq_u64(window, expected) };

        // Each lane is all-ones on equality → vgetq_lane_u64 returns u64::MAX.
        // SAFETY: `vgetq_lane_u64` is a pure register operation; NEON is enabled by `#[target_feature]`.
        if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
            return false;
        }

        i += 2;
    }

    // Scalar tail for the last word (when len is odd).
    while i < len {
        let w0 = src[i];
        let w1 = src[i + 1];
        if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
            return false;
        }
        i += 1;
    }

    true
}
