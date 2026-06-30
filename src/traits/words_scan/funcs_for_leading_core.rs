//! Leading value-bit count — forward scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.

use super::funcs_for_chunk_eq::{LANES, LANES_2X, chunk_eq, chunk_eq_2x};
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use super::funcs_for_chunk_eq::{chunk_eq_2x_aligned, chunk_eq_aligned};

use crate::{SMALL_WORDS, WORD_BITS, low_mask};

#[inline]
fn count_trailing<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.trailing_zeros() as usize
    } else {
        (!val).trailing_zeros() as usize
    }
}

const ALIGN_THRESHOLD: usize = 128;

#[inline(always)]
pub(super) fn leading<const FILL: u64, const WORD_ALIGNED: bool>(
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
            let base = unsafe { bits.as_ptr().add(wi) };
            let end = unsafe { base.add(total) };

            // First-word fast path — catches early non-FILL.
            let w0 = unsafe { *base };
            if w0 != FILL {
                return (scanned + count_trailing::<FILL>(w0)).min(bit_len);
            }
            // Start SIMD from `base` (not base+1).  Word 0 is
            // double‑checked (fast path + SIMD) but this keeps
            // the iteration count a clean multiple of STRIDE.
            let mut p = base;

            // ── SIMD scan ─────────────────────────────────────────
            // Raw SIMD intrinsics are inlined directly here (not
            // behind a #[target_feature] call gate) so LLVM can
            // fully inline through the entire call chain.
            //
            // AVX2
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            unsafe {
                #[cfg(target_arch = "x86")]
                use core::arch::x86::{
                    __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x,
                    _mm256_testz_si256, _mm256_xor_si256,
                };
                #[cfg(target_arch = "x86_64")]
                use core::arch::x86_64::{
                    __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x,
                    _mm256_testz_si256, _mm256_xor_si256,
                };
                const LANES: usize = 4;
                const STRIDE: usize = 8;

                // Inline helper for the unaligned 2× check.
                macro_rules! is_all_fill_2x {
                    ($ptr:expr) => {
                        if FILL == 0 {
                            let d0 = _mm256_loadu_si256($ptr.cast::<__m256i>());
                            let d1 = _mm256_loadu_si256($ptr.add(LANES).cast::<__m256i>());
                            _mm256_testz_si256(d0, d0) != 0 && _mm256_testz_si256(d1, d1) != 0
                        } else {
                            let fill_vec = _mm256_set1_epi64x(FILL as i64);
                            let d0 = _mm256_loadu_si256($ptr.cast::<__m256i>());
                            let x0 = _mm256_xor_si256(d0, fill_vec);
                            let d1 = _mm256_loadu_si256($ptr.add(LANES).cast::<__m256i>());
                            let x1 = _mm256_xor_si256(d1, fill_vec);
                            _mm256_testz_si256(x0, x0) != 0 && _mm256_testz_si256(x1, x1) != 0
                        }
                    };
                }
                // Inline helper for the aligned 2× check.
                macro_rules! is_all_fill_2x_aligned {
                    ($ptr:expr) => {
                        if FILL == 0 {
                            let d0 = _mm256_load_si256($ptr.cast::<__m256i>());
                            let d1 = _mm256_load_si256($ptr.add(LANES).cast::<__m256i>());
                            _mm256_testz_si256(d0, d0) != 0 && _mm256_testz_si256(d1, d1) != 0
                        } else {
                            let fill_vec = _mm256_set1_epi64x(FILL as i64);
                            let d0 = _mm256_load_si256($ptr.cast::<__m256i>());
                            let x0 = _mm256_xor_si256(d0, fill_vec);
                            let d1 = _mm256_load_si256($ptr.add(LANES).cast::<__m256i>());
                            let x1 = _mm256_xor_si256(d1, fill_vec);
                            _mm256_testz_si256(x0, x0) != 0 && _mm256_testz_si256(x1, x1) != 0
                        }
                    };
                }

                if total >= ALIGN_THRESHOLD {
                    let misalign = (base as usize % 32) / 8;
                    if misalign > 0 {
                        let prefix_end = base.add(misalign);
                        while p < prefix_end {
                            if *p != FILL {
                                let off = (p as usize - base as usize) / 8;
                                return (scanned + off * WORD_BITS + count_trailing::<FILL>(*p))
                                    .min(bit_len);
                            }
                            p = p.add(1);
                        }
                    }
                    let mut iters =
                        (end as usize - p as usize) / (STRIDE * core::mem::size_of::<u64>());
                    while iters > 0 {
                        if !is_all_fill_2x_aligned!(p) {
                            break;
                        }
                        p = p.add(STRIDE);
                        iters -= 1;
                    }
                } else {
                    let mut iters = total / STRIDE;
                    while iters > 0 {
                        if !is_all_fill_2x!(p) {
                            break;
                        }
                        p = p.add(STRIDE);
                        iters -= 1;
                    }
                }
            }

            // SSE2
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse2",
                not(target_feature = "avx2")
            ))]
            unsafe {
                let mut iters = total / LANES_2X;
                while iters > 0 {
                    if !chunk_eq_2x::<FILL>(p) {
                        break;
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
            }

            // NEON
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            unsafe {
                let mut iters = total / LANES_2X;
                while iters > 0 {
                    if !chunk_eq_2x::<FILL>(p) {
                        break;
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
            }

            // Scalar
            #[cfg(not(any(
                all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    any(target_feature = "avx2", target_feature = "sse2")
                ),
                all(target_arch = "aarch64", target_feature = "neon"),
            )))]
            unsafe {
                let limit = end.sub(LANES);
                while p <= limit {
                    if !chunk_eq::<FILL>(p) {
                        break;
                    }
                    p = p.add(LANES);
                }
            }

            let done_words = (p as usize - base as usize) / 8;
            scanned += done_words * WORD_BITS;

            if (p as usize) >= (end as usize) && end_rem == 0 {
                return scanned.min(bit_len);
            }

            let rem = (end as usize - p as usize) / 8;
            for _ in 0..rem {
                unsafe {
                    if *p != FILL {
                        scanned += count_trailing::<FILL>(*p);
                        wi = mid_end;
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
