//! Trailing value-bit count — reverse scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.
//! When `WORD_ALIGNED` is `true` the caller guarantees `start_offset == 0`,
//! allowing the compiler to eliminate the first-word LZCNT phase.

use crate::{SMALL_WORDS, WORD_BITS};

// ── Scalar helper ──────────────────────────────────────────────────────

/// Counts leading bits within a single u64 word that match `FILL`.
#[inline]
fn count_leading<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.leading_zeros() as usize
    } else {
        (!val).leading_zeros() as usize
    }
}

// ── Dispatch ───────────────────────────────────────────────────────────

#[inline]
pub(crate) fn trailing<const FILL: u64, const WORD_ALIGNED: bool>(
    bits: &[u64],
    start_offset: u32,
    bit_len: usize,
) -> usize {
    if bit_len == 0 {
        return 0;
    }

    let end_offset = start_offset as usize + bit_len;
    let end_rem = end_offset % WORD_BITS;
    let last_wi = (end_offset - 1) / WORD_BITS;

    let mut scanned = 0usize;

    // ── Last partial word ─────────────────────────────────────────
    if end_rem != 0 {
        let last_limit = if last_wi == 0 {
            end_rem - start_offset as usize
        } else {
            end_rem
        };
        let shifted = bits[last_wi] << (WORD_BITS - end_rem);
        let last_count = count_leading::<FILL>(shifted).min(last_limit);
        if last_count < last_limit {
            return last_count;
        }
        scanned += last_limit;
        if last_wi == 0 {
            return scanned.min(bit_len);
        }
    }

    // ── Full middle words — reverse SIMD scan ────────────────────
    let wi_end = if end_rem != 0 { last_wi - 1 } else { last_wi };
    let mid_first = if !WORD_ALIGNED && start_offset > 0 {
        1
    } else {
        0
    };

    if wi_end >= mid_first {
        let total_words = wi_end + 1 - mid_first;
        let ptr = bits.as_ptr();

        let mut done = 0usize;

        // ── Rightmost-word fast path ─────────────────────────────
        // Early exit if the answer is in the rightmost full word,
        // without disrupting the SIMD stride alignment.
        {
            let w = bits[wi_end];
            if w != FILL {
                scanned += count_leading::<FILL>(w);
                return scanned.min(bit_len);
            }
        }

        // ── Tiny inputs — simple scalar reverse scan ────────────
        if total_words < SMALL_WORDS {
            while done < total_words {
                let wi = wi_end - done;
                if bits[wi] != FILL {
                    scanned += count_leading::<FILL>(bits[wi]);
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                done += 1;
            }
            // All full words match FILL — skip SIMD.
        } else {
            // ── Default: runtime SIMD detection ───────────────────
            #[cfg(not(feature = "compile-time-dispatch"))]
            {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    let done_before = done;
                    if crate::cpuid::features().avx2 {
                        // SAFETY: CPUID confirmed AVX2 is available.
                        done =
                            unsafe { avx2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    } else {
                        // SAFETY: SSE2 is baseline on x86-64.
                        done =
                            unsafe { sse2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    }
                    scanned += (done - done_before) * WORD_BITS;
                }
                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    let done_before = done;
                    // SAFETY: NEON is available per `#[cfg]` gate.
                    done = unsafe { neon::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    scanned += (done - done_before) * WORD_BITS;
                }
                #[allow(unused)]
                {
                    // Scalar fallback: `done` stays unchanged; shared
                    // tail below scans word-by-word.
                }
            }

            // ── compile-time-dispatch: pure #[cfg] cascade ────────
            #[cfg(feature = "compile-time-dispatch")]
            {
                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    target_feature = "avx2"
                ))]
                {
                    let done_before = done;
                    // SAFETY: AVX2 is guaranteed by compile-time gate.
                    done = unsafe { avx2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    scanned += (done - done_before) * WORD_BITS;
                }

                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    any(target_feature = "sse2", target_feature = "ssse3"),
                    not(target_feature = "avx2")
                ))]
                {
                    let done_before = done;
                    // SAFETY: SSE2 is guaranteed by compile-time gate.
                    done = unsafe { sse2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    scanned += (done - done_before) * WORD_BITS;
                }

                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    let done_before = done;
                    // SAFETY: NEON is guaranteed by compile-time gate.
                    done = unsafe { neon::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                    scanned += (done - done_before) * WORD_BITS;
                }

                #[allow(unused)]
                {
                    // Scalar fallback.
                }
            }
        } // else (SIMD path)

        // ── Scalar tail ──────────────────────────────────────────
        while done < total_words {
            let wi = wi_end - done;
            if bits[wi] != FILL {
                scanned += count_leading::<FILL>(bits[wi]);
                return scanned.min(bit_len);
            }
            scanned += WORD_BITS;
            done += 1;
        }
    }

    // ── First-word partial (trailing side) ───────────────────────
    if !WORD_ALIGNED && start_offset > 0 {
        let first_limit = WORD_BITS - start_offset as usize;
        let first_count = count_leading::<FILL>(bits[0]).min(first_limit);
        scanned += first_count;
    }

    scanned.min(bit_len)
}

// ═══════════════════════════════════════════════════════════════════════
// AVX2 backend — 256-bit / 4-lane, unaligned loads only.
// ═══════════════════════════════════════════════════════════════════════

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };

    const LANES: usize = 4;
    const STRIDE: usize = 8;

    /// AVX2 reverse scan: scans backwards from `wi_end` and advances `done`
    /// past all-FILL 256-bit chunks.
    ///
    /// Returns the updated `done` count (total words consumed from right).
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available (checked via CPUID).
    /// `ptr` through `ptr.add(wi_end + 1)` must be valid for u64 reads.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn trailing_scan<const FILL: u64>(
        ptr: *const u64,
        wi_end: usize,
        mut done: usize,
        total_words: usize,
    ) -> usize {
        // SAFETY: only callable when AVX2 is available (caller verified
        // via CPUID).  All pointer arithmetic stays within bounds.
        unsafe {
            // 2×‑unrolled
            while done + STRIDE <= total_words {
                let chunk_start = wi_end + 1 - (done + STRIDE);
                let d0_ok = if FILL == 0 {
                    let d = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                    _mm256_testz_si256(d, d) != 0
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                    let x = _mm256_xor_si256(d, fill_vec);
                    _mm256_testz_si256(x, x) != 0
                };
                let d1_ok = if FILL == 0 {
                    let d = _mm256_loadu_si256(ptr.add(chunk_start + LANES).cast::<__m256i>());
                    _mm256_testz_si256(d, d) != 0
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d = _mm256_loadu_si256(ptr.add(chunk_start + LANES).cast::<__m256i>());
                    let x = _mm256_xor_si256(d, fill_vec);
                    _mm256_testz_si256(x, x) != 0
                };
                if !d0_ok || !d1_ok {
                    break;
                }
                done += STRIDE;
            }
            // Single-chunk remainder
            while done + LANES <= total_words {
                let chunk_start = wi_end + 1 - (done + LANES);
                if FILL == 0 {
                    let d = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                    if _mm256_testz_si256(d, d) == 0 {
                        break;
                    }
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                    let x = _mm256_xor_si256(d, fill_vec);
                    if _mm256_testz_si256(x, x) == 0 {
                        break;
                    }
                }
                done += LANES;
            }

            done
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SSE2 backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
        _mm_setzero_si128, _mm_xor_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
        _mm_setzero_si128, _mm_xor_si128,
    };

    const LANES: usize = 2;
    const LANES_2X: usize = LANES * 2;

    #[inline(always)]
    unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller ensures `ptr` is valid for 2 u64 reads and
        // SSE2 is available.
        unsafe {
            let data = _mm_loadu_si128(ptr.cast::<__m128i>());
            if FILL == 0 {
                let cmp = _mm_cmpeq_epi32(data, _mm_setzero_si128());
                _mm_movemask_epi8(cmp) == 0xFFFF
            } else {
                let fill_vec = _mm_set1_epi64x(FILL as i64);
                let xor = _mm_xor_si128(data, fill_vec);
                let cmp = _mm_cmpeq_epi32(xor, _mm_setzero_si128());
                _mm_movemask_epi8(cmp) == 0xFFFF
            }
        }
    }

    /// SSE2 reverse scan: advances `done` past all-FILL chunks from the right.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE2 is available (baseline on x86-64).
    /// `ptr` through `ptr.add(wi_end + 1)` must be valid for u64 reads.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn trailing_scan<const FILL: u64>(
        ptr: *const u64,
        wi_end: usize,
        mut done: usize,
        total_words: usize,
    ) -> usize {
        // SAFETY: only callable when SSE2 is available (caller verified
        // via CPUID, or SSE2 is baseline).
        unsafe {
            while done + LANES_2X <= total_words {
                let chunk_start = wi_end + 1 - (done + LANES_2X);
                if !chunk_eq::<FILL>(ptr.add(chunk_start))
                    || !chunk_eq::<FILL>(ptr.add(chunk_start + LANES))
                {
                    return done;
                }
                done += LANES_2X;
            }
            while done + LANES <= total_words {
                let chunk_start = wi_end + 1 - (done + LANES);
                if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                    break;
                }
                done += LANES;
            }
            done
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEON backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::arch::aarch64::{vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

    const LANES: usize = 2;
    const LANES_2X: usize = LANES * 2;

    #[inline(always)]
    unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller ensures `ptr` is valid for 2 u64 reads and
        // NEON is available.
        unsafe {
            let data = vld1q_u64(ptr);
            let cmp = vceqq_u64(data, vdupq_n_u64(FILL));
            vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
        }
    }

    /// NEON reverse scan: advances `done` past all-FILL chunks from the right.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    /// `ptr` through `ptr.add(wi_end + 1)` must be valid for u64 reads.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn trailing_scan<const FILL: u64>(
        ptr: *const u64,
        wi_end: usize,
        mut done: usize,
        total_words: usize,
    ) -> usize {
        // SAFETY: only callable when NEON is available.  All pointer
        // arithmetic stays within bounds.
        unsafe {
            while done + LANES_2X <= total_words {
                let chunk_start = wi_end + 1 - (done + LANES_2X);
                if !chunk_eq::<FILL>(ptr.add(chunk_start))
                    || !chunk_eq::<FILL>(ptr.add(chunk_start + LANES))
                {
                    return done;
                }
                done += LANES_2X;
            }
            while done + LANES <= total_words {
                let chunk_start = wi_end + 1 - (done + LANES);
                if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                    break;
                }
                done += LANES;
            }
            done
        }
    }
}
