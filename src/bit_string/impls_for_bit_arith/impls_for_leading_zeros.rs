use crate::traits::WordsScan;
use crate::{SMALL_WORDS, WORD_BITS, low_mask};

use super::BitString;

impl BitString {
    /// Returns the number of consecutive `false` bits from the start.
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };

        // Tiny inputs — unrolled scalar, no SIMD or loop overhead.
        if mid_end < SMALL_WORDS {
            match mid_end {
                0 => {
                    let last = unsafe { *words_ptr } & low_mask(end_rem);
                    if last == 0 {
                        return bit_len;
                    }
                    return (last.trailing_zeros() as usize).min(bit_len);
                }
                1 => {
                    let w0 = unsafe { *words_ptr };
                    if w0 != 0 {
                        return (w0.trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(1) } & low_mask(end_rem);
                    if last == 0 {
                        return bit_len;
                    }
                    return (64 + last.trailing_zeros() as usize).min(bit_len);
                }
                2 => {
                    let w0 = unsafe { *words_ptr };
                    if w0 != 0 {
                        return (w0.trailing_zeros() as usize).min(bit_len);
                    }
                    let w1 = unsafe { *words_ptr.add(1) };
                    if w1 != 0 {
                        return (64 + w1.trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(2) } & low_mask(end_rem);
                    if last == 0 {
                        return bit_len;
                    }
                    return (128 + last.trailing_zeros() as usize).min(bit_len);
                }
                _ => {
                    // 3 full words (mid_end == 3, SMALL_WORDS == 4 for AVX2)
                    let w0 = unsafe { *words_ptr };
                    if w0 != 0 {
                        return (w0.trailing_zeros() as usize).min(bit_len);
                    }
                    let w1 = unsafe { *words_ptr.add(1) };
                    if w1 != 0 {
                        return (64 + w1.trailing_zeros() as usize).min(bit_len);
                    }
                    let w2 = unsafe { *words_ptr.add(2) };
                    if w2 != 0 {
                        return (128 + w2.trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(3) } & low_mask(end_rem);
                    if last == 0 {
                        return bit_len;
                    }
                    return (192 + last.trailing_zeros() as usize).min(bit_len);
                }
            }
        }

        // ── First-word fast path ─────────────────────────────────
        // Catches the common case where the answer lies in word 0.
        {
            let w0 = unsafe { *words_ptr };
            if w0 != 0 {
                return (w0.trailing_zeros() as usize).min(bit_len);
            }
        }

        // ── SIMD countdown ──────────────────────────────────────
        // Exactly one cfg block is compiled; each sets `scanned`.

        let mut scanned: usize;

        // AVX2  (256‑bit, 4 × u64)
        // For large inputs (≥ 128 words), the alignment prefix +
        // aligned `vmovdqa` wins after ~32 SIMD iterations amortise
        // the 4‑scalar‑word overhead.  For medium inputs, unaligned
        // `vmovdqu` avoids the prefix/remainder cost entirely.
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            use core::arch::x86_64::{
                __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_testz_si256,
            };
            const LANES: usize = 4;
            // Threshold — above this many full words the alignment prefix
            // is worth its scalar-overhead cost.
            const ALIGN_THRESHOLD: usize = 128;

            let end = unsafe { words_ptr.add(mid_end) };
            let mut p = words_ptr;

            if mid_end >= ALIGN_THRESHOLD {
                // Align p to 32‑byte boundary for aligned `vmovdqa`.
                let misalign = (p as usize % 32) / 8;
                if misalign > 0 {
                    let prefix_end = unsafe { p.add(misalign) };
                    while p < prefix_end {
                        let w = unsafe { *p };
                        if w != 0 {
                            let tz = w.trailing_zeros() as usize;
                            return tz.min(bit_len);
                        }
                        p = unsafe { p.add(1) };
                    }
                }

                let mut iters = (end as usize - p as usize) / (LANES * core::mem::size_of::<u64>());
                while iters > 0 {
                    let all_zero = unsafe {
                        let data = _mm256_load_si256(p.cast::<__m256i>());
                        _mm256_testz_si256(data, data) != 0
                    };
                    if !all_zero {
                        break;
                    }
                    p = unsafe { p.add(LANES) };
                    iters -= 1;
                }
            } else {
                // Unaligned — no prefix, SIMD loop covers everything.
                let mut iters = mid_end / LANES;
                while iters > 0 {
                    let all_zero = unsafe {
                        let data = _mm256_loadu_si256(p.cast::<__m256i>());
                        _mm256_testz_si256(data, data) != 0
                    };
                    if !all_zero {
                        break;
                    }
                    p = unsafe { p.add(LANES) };
                    iters -= 1;
                }
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            if (p as usize) >= (end as usize) && end_rem == 0 {
                return scanned;
            }

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != 0 {
                    scanned += w.trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += last.trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        // SSE2  (128‑bit, 2 × u64)
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            use core::arch::x86_64::{
                __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_setzero_si128,
            };
            const LANES: usize = 2;

            let end = unsafe { words_ptr.add(mid_end) };
            let limit = unsafe { end.sub(LANES) };
            let zero = unsafe { _mm_setzero_si128() };
            let mut p = words_ptr;

            while p <= limit {
                let all_zero = unsafe {
                    let data = _mm_loadu_si128(p.cast::<__m128i>());
                    let cmp = _mm_cmpeq_epi32(data, zero);
                    _mm_movemask_epi8(cmp) == 0xFFFF
                };
                if !all_zero {
                    break;
                }
                p = unsafe { p.add(LANES) };
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != 0 {
                    scanned += w.trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += last.trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        // NEON  (aarch64, 128‑bit, 2 × u64)
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            use core::arch::aarch64::{vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};
            const LANES: usize = 2;

            let end = unsafe { words_ptr.add(mid_end) };
            let limit = unsafe { end.sub(LANES) };
            let mut p = words_ptr;

            while p <= limit {
                let all_zero = unsafe {
                    let data = vld1q_u64(p);
                    let cmp = vceqq_u64(data, vdupq_n_u64(0));
                    vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
                };
                if !all_zero {
                    break;
                }
                p = unsafe { p.add(LANES) };
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != 0 {
                    scanned += w.trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += last.trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        // Scalar fallback.
        #[cfg(not(any(
            all(
                target_arch = "x86_64",
                any(target_feature = "avx2", target_feature = "sse2")
            ),
            all(target_arch = "aarch64", target_feature = "neon"),
        )))]
        {
            let end = unsafe { words_ptr.add(mid_end) };
            scanned = 0;
            let mut p = words_ptr;

            while p < end {
                let w = unsafe { *p };
                if w != 0 {
                    scanned += w.trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += last.trailing_zeros() as usize;
            }
            scanned.min(bit_len)
        }
    }

    /// Returns the number of consecutive `true` bits from the start.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };

        if mid_end < SMALL_WORDS {
            match mid_end {
                0 => {
                    let last = unsafe { *words_ptr } & low_mask(end_rem);
                    if last == low_mask(end_rem) {
                        return bit_len;
                    }
                    return ((!last).trailing_zeros() as usize).min(bit_len);
                }
                1 => {
                    let w0 = unsafe { *words_ptr };
                    if w0 != u64::MAX {
                        return ((!w0).trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(1) } & low_mask(end_rem);
                    if last == low_mask(end_rem) {
                        return bit_len;
                    }
                    return (64 + (!last).trailing_zeros() as usize).min(bit_len);
                }
                2 => {
                    let w0 = unsafe { *words_ptr };
                    if w0 != u64::MAX {
                        return ((!w0).trailing_zeros() as usize).min(bit_len);
                    }
                    let w1 = unsafe { *words_ptr.add(1) };
                    if w1 != u64::MAX {
                        return (64 + (!w1).trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(2) } & low_mask(end_rem);
                    if last == low_mask(end_rem) {
                        return bit_len;
                    }
                    return (128 + (!last).trailing_zeros() as usize).min(bit_len);
                }
                _ => {
                    // 3 full words
                    let w0 = unsafe { *words_ptr };
                    if w0 != u64::MAX {
                        return ((!w0).trailing_zeros() as usize).min(bit_len);
                    }
                    let w1 = unsafe { *words_ptr.add(1) };
                    if w1 != u64::MAX {
                        return (64 + (!w1).trailing_zeros() as usize).min(bit_len);
                    }
                    let w2 = unsafe { *words_ptr.add(2) };
                    if w2 != u64::MAX {
                        return (128 + (!w2).trailing_zeros() as usize).min(bit_len);
                    }
                    if end_rem == 0 {
                        return bit_len;
                    }
                    let last = unsafe { *words_ptr.add(3) } & low_mask(end_rem);
                    if last == low_mask(end_rem) {
                        return bit_len;
                    }
                    return (192 + (!last).trailing_zeros() as usize).min(bit_len);
                }
            }
        }

        let mut scanned: usize;

        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            use core::arch::x86_64::{
                __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
                _mm256_xor_si256,
            };
            const LANES: usize = 4;

            let end = unsafe { words_ptr.add(mid_end) };
            let limit = unsafe { end.sub(LANES) };
            let fill = unsafe { _mm256_set1_epi64x(-1) };
            let mut p = words_ptr;

            while p <= limit {
                let all_ones = unsafe {
                    let data = _mm256_loadu_si256(p.cast::<__m256i>());
                    let xor = _mm256_xor_si256(data, fill);
                    _mm256_testz_si256(xor, xor) != 0
                };
                if !all_ones {
                    break;
                }
                p = unsafe { p.add(LANES) };
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != u64::MAX {
                    scanned += (!w).trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += (!last).trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            use core::arch::x86_64::{
                __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
                _mm_setzero_si128, _mm_xor_si128,
            };
            const LANES: usize = 2;

            let end = unsafe { words_ptr.add(mid_end) };
            let limit = unsafe { end.sub(LANES) };
            let zero = unsafe { _mm_setzero_si128() };
            let fill_vec = unsafe { _mm_set1_epi64x(-1) };
            let mut p = words_ptr;

            while p <= limit {
                let all_ones = unsafe {
                    let data = _mm_loadu_si128(p.cast::<__m128i>());
                    let xor = _mm_xor_si128(data, fill_vec);
                    let cmp = _mm_cmpeq_epi32(xor, zero);
                    _mm_movemask_epi8(cmp) == 0xFFFF
                };
                if !all_ones {
                    break;
                }
                p = unsafe { p.add(LANES) };
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != u64::MAX {
                    scanned += (!w).trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += (!last).trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            use core::arch::aarch64::{vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};
            const LANES: usize = 2;

            let end = unsafe { words_ptr.add(mid_end) };
            let limit = unsafe { end.sub(LANES) };
            let fill = u64::MAX;
            let mut p = words_ptr;

            while p <= limit {
                let all_ones = unsafe {
                    let data = vld1q_u64(p);
                    let cmp = vceqq_u64(data, vdupq_n_u64(fill));
                    vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
                };
                if !all_ones {
                    break;
                }
                p = unsafe { p.add(LANES) };
            }

            let done = (p as usize - words_ptr as usize) / 8;
            scanned = done * WORD_BITS;

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                let w = unsafe { *p };
                if w != u64::MAX {
                    scanned += (!w).trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += (!last).trailing_zeros() as usize;
            }
            return scanned.min(bit_len);
        }

        #[cfg(not(any(
            all(
                target_arch = "x86_64",
                any(target_feature = "avx2", target_feature = "sse2")
            ),
            all(target_arch = "aarch64", target_feature = "neon"),
        )))]
        {
            let end = unsafe { words_ptr.add(mid_end) };
            scanned = 0;
            let mut p = words_ptr;

            while p < end {
                let w = unsafe { *p };
                if w != u64::MAX {
                    scanned += (!w).trailing_zeros() as usize;
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                p = unsafe { p.add(1) };
            }
            if end_rem != 0 {
                let last = unsafe { *p } & low_mask(end_rem);
                scanned += (!last).trailing_zeros() as usize;
            }
            scanned.min(bit_len)
        }
    }

    /// Returns the number of consecutive `false` bits from the end.
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        use crate::FILL_ZEROS;
        self.words()
            .trailing_value_bits::<FILL_ZEROS, true>(0, self.bit_len)
    }

    /// Returns the number of consecutive `true` bits from the end.
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        use crate::FILL_ONES;
        self.words()
            .trailing_value_bits::<FILL_ONES, true>(0, self.bit_len)
    }
}
