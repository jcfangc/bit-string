//! Leading value-bit count — forward scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.

use crate::{SMALL_WORDS, WORD_BITS, low_mask};

// ── Scalar helper ──────────────────────────────────────────────────────

#[inline]
fn count_trailing<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.trailing_zeros() as usize
    } else {
        (!val).trailing_zeros() as usize
    }
}

// ── Dispatch ───────────────────────────────────────────────────────────

#[inline]
pub(crate) fn leading<const FILL: u64, const WORD_ALIGNED: bool>(
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
    let mut wi = 0usize;

    // ── Unaligned first word ───────────────────────────────────────
    if !WORD_ALIGNED && start_offset != 0 {
        let first_val = bits[0] >> start_offset;
        let first_limit = (WORD_BITS - start_offset as usize).min(bit_len);
        let first_count = count_trailing::<FILL>(first_val).min(first_limit);
        if first_count < first_limit {
            return first_count;
        }
        scanned += first_limit;
        wi = 1;
    }

    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let total = mid_end - wi;

        // ── Tiny inputs: scalar ────────────────────────────────────
        if total < SMALL_WORDS {
            for i in 0..total {
                let w = bits[wi + i];
                if w != FILL {
                    return (scanned + count_trailing::<FILL>(w)).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            wi = mid_end;
        } else {
            // SAFETY: `wi < mid_end` and `total = mid_end - wi`,
            // so `bits[wi..mid_end]` is within the input slice.
            let base = unsafe { bits.as_ptr().add(wi) };
            // SAFETY: `end` is one past the last word — used only as a
            // limit pointer, never dereferenced.
            let end = unsafe { base.add(total) };

            // First-word fast path — catches early non-FILL.
            // SAFETY: `total > 0` (we are in the `total >= SMALL_WORDS`
            // branch), so `base` is valid for at least one u64 read.
            let w0 = unsafe { *base };
            if w0 != FILL {
                return (scanned + count_trailing::<FILL>(w0)).min(bit_len);
            }
            // Start SIMD from `base` (not base+1).  Word 0 is
            // double-checked (fast path + SIMD) but this keeps the
            // iteration count a clean multiple of the SIMD stride.
            let mut p = base;

            // ── Default: runtime SIMD detection ─────────────────────
            #[cfg(not(feature = "compile-time-dispatch"))]
            {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if crate::cpuid::features().avx2 {
                    // SAFETY: CPUID confirmed AVX2 is available.
                    // `p` through `end` are within the input slice.
                    p = unsafe { avx2::leading_scan::<FILL>(p, end, base, total) };
                } else {
                    // SAFETY: SSE2 is baseline on x86-64.
                    // `p` through `end` are within the input slice.
                    p = unsafe { sse2::leading_scan::<FILL>(p, end, total) };
                }
                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    // SAFETY: NEON is available per `#[cfg]` gate.
                    // `p` through `end` are within the input slice.
                    p = unsafe { neon::leading_scan::<FILL>(p, end, total) };
                }
                #[allow(unused)]
                {
                    // Scalar fallback: `p` stays at base; shared tail
                    // below scans word-by-word.
                }
            }

            // ── compile-time-dispatch: pure #[cfg] cascade ──────────
            #[cfg(feature = "compile-time-dispatch")]
            {
                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    target_feature = "avx2"
                ))]
                {
                    // SAFETY: AVX2 is guaranteed by compile-time
                    // `#[cfg]` gate.
                    p = unsafe { avx2::leading_scan::<FILL>(p, end, base, total) };
                }

                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    any(target_feature = "sse2", target_feature = "ssse3"),
                    not(target_feature = "avx2")
                ))]
                {
                    // SAFETY: SSE2 is guaranteed by compile-time
                    // `#[cfg]` gate.
                    p = unsafe { sse2::leading_scan::<FILL>(p, end, total) };
                }

                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    // SAFETY: NEON is guaranteed by compile-time
                    // `#[cfg]` gate.
                    p = unsafe { neon::leading_scan::<FILL>(p, end, total) };
                }

                #[allow(unused)]
                {
                    // Scalar fallback: `p` stays at base.
                }
            }

            // ── Post-SIMD: shared scalar remainder ─────────────────
            let done_words = (p as usize - base as usize) / 8;
            scanned += done_words * WORD_BITS;

            if (p as usize) >= (end as usize) && end_rem == 0 {
                return scanned.min(bit_len);
            }

            let rem = (end as usize - p as usize) / 8;
            // SAFETY: `rem` is computed from `end - p`, so `p` through
            // `p.add(rem - 1)` lies within `[base, end)`.
            for _ in 0..rem {
                unsafe {
                    if *p != FILL {
                        scanned += count_trailing::<FILL>(*p);
                        return (scanned).min(bit_len);
                    }
                    scanned += WORD_BITS;
                    p = p.add(1);
                }
            }
            wi = mid_end;
        }
    }

    if end_rem != 0 && wi == last_wi {
        let last_val = bits[wi] & low_mask(end_rem);
        scanned += count_trailing::<FILL>(last_val).min(end_rem);
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
        __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
        _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
        _mm256_xor_si256,
    };

    const LANES: usize = 4;
    const STRIDE: usize = 8; // 2 × LANES for unrolled iteration
    const ALIGN_THRESHOLD: usize = 128;

    /// AVX2 forward scan: advances `p` past all-FILL 256-bit chunks.
    ///
    /// For large inputs (≥ ALIGN_THRESHOLD words), aligns `p` to a
    /// 32-byte boundary and uses `_mm256_load_si256` (aligned) for the
    /// hot loop.  Smaller inputs use `_mm256_loadu_si256` (unaligned)
    /// to avoid the alignment prefix overhead.
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available (checked via CPUID).
    /// `p` through `end` must be valid for u64 reads.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn leading_scan<const FILL: u64>(
        mut p: *const u64,
        end: *const u64,
        base: *const u64,
        total: usize,
    ) -> *const u64 {
        // SAFETY: only callable when AVX2 is available (caller verified
        // via CPUID).  All pointer arithmetic stays within bounds.
        unsafe {
            if total >= ALIGN_THRESHOLD {
                // Distance in words to the next 32-byte boundary.
                let words_to_align = (4usize.wrapping_sub((base as usize / 8) % 4)) % 4;
                if words_to_align > 0 {
                    let prefix_end = base.add(words_to_align);
                    while p < prefix_end {
                        if *p != FILL {
                            return p;
                        }
                        p = p.add(1);
                    }
                }
                let mut iters =
                    (end as usize - p as usize) / (STRIDE * core::mem::size_of::<u64>());
                while iters > 0 {
                    if FILL == 0 {
                        let d0 = _mm256_load_si256(p.cast::<__m256i>());
                        let d1 = _mm256_load_si256(p.add(LANES).cast::<__m256i>());
                        if _mm256_testz_si256(d0, d0) == 0 || _mm256_testz_si256(d1, d1) == 0 {
                            break;
                        }
                    } else {
                        let fill_vec = _mm256_set1_epi64x(FILL as i64);
                        let d0 = _mm256_load_si256(p.cast::<__m256i>());
                        let x0 = _mm256_xor_si256(d0, fill_vec);
                        let d1 = _mm256_load_si256(p.add(LANES).cast::<__m256i>());
                        let x1 = _mm256_xor_si256(d1, fill_vec);
                        if _mm256_testz_si256(x0, x0) == 0 || _mm256_testz_si256(x1, x1) == 0 {
                            break;
                        }
                    }
                    p = p.add(STRIDE);
                    iters -= 1;
                }
            } else {
                // 2×-unrolled unaligned path.
                let mut iters = total / STRIDE;
                while iters > 0 {
                    if FILL == 0 {
                        let d0 = _mm256_loadu_si256(p.cast::<__m256i>());
                        let d1 = _mm256_loadu_si256(p.add(LANES).cast::<__m256i>());
                        if _mm256_testz_si256(d0, d0) == 0 || _mm256_testz_si256(d1, d1) == 0 {
                            break;
                        }
                    } else {
                        let fill_vec = _mm256_set1_epi64x(FILL as i64);
                        let d0 = _mm256_loadu_si256(p.cast::<__m256i>());
                        let x0 = _mm256_xor_si256(d0, fill_vec);
                        let d1 = _mm256_loadu_si256(p.add(LANES).cast::<__m256i>());
                        let x1 = _mm256_xor_si256(d1, fill_vec);
                        if _mm256_testz_si256(x0, x0) == 0 || _mm256_testz_si256(x1, x1) == 0 {
                            break;
                        }
                    }
                    p = p.add(STRIDE);
                    iters -= 1;
                }
            }
            // Single-chunk remainder (LANES = 4 words).
            let limit = end.sub(LANES);
            while p <= limit {
                if FILL == 0 {
                    let d = _mm256_loadu_si256(p.cast::<__m256i>());
                    if _mm256_testz_si256(d, d) == 0 {
                        break;
                    }
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d = _mm256_loadu_si256(p.cast::<__m256i>());
                    let x = _mm256_xor_si256(d, fill_vec);
                    if _mm256_testz_si256(x, x) == 0 {
                        break;
                    }
                }
                p = p.add(LANES);
            }
            p
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

    /// SSE2 forward scan: advances `p` past all-FILL chunks.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE2 is available (baseline on x86-64).
    /// `p` through `end` must be valid for u64 reads.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn leading_scan<const FILL: u64>(
        mut p: *const u64,
        end: *const u64,
        total: usize,
    ) -> *const u64 {
        // SAFETY: only callable when SSE2 is available (caller verified
        // via CPUID, or SSE2 is baseline).
        unsafe {
            let mut iters = total / LANES_2X;
            while iters > 0 {
                if !chunk_eq::<FILL>(p) || !chunk_eq::<FILL>(p.add(LANES)) {
                    return p;
                }
                p = p.add(LANES_2X);
                iters -= 1;
            }
            let limit = end.sub(LANES);
            while p <= limit {
                if !chunk_eq::<FILL>(p) {
                    break;
                }
                p = p.add(LANES);
            }
            p
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEON backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

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

    /// NEON forward scan: advances `p` past all-FILL chunks.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    /// `p` through `end` must be valid for u64 reads.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn leading_scan<const FILL: u64>(
        mut p: *const u64,
        end: *const u64,
        total: usize,
    ) -> *const u64 {
        // SAFETY: only callable when NEON is available.  All pointer
        // arithmetic stays within `[p, end)`.
        unsafe {
            let mut iters = total / LANES_2X;
            while iters > 0 {
                if !chunk_eq::<FILL>(p) || !chunk_eq::<FILL>(p.add(LANES)) {
                    return p;
                }
                p = p.add(LANES_2X);
                iters -= 1;
            }
            let limit = end.sub(LANES);
            while p <= limit {
                if !chunk_eq::<FILL>(p) {
                    break;
                }
                p = p.add(LANES);
            }
            p
        }
    }
}
