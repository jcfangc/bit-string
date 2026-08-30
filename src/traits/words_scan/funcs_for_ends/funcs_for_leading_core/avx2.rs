// ═══════════════════════════════════════════════════════════════════════
// AVX2 backend — 256-bit / 4-lane, unaligned loads only.
// ═══════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_testz_si256, _mm256_xor_si256,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_testz_si256, _mm256_xor_si256,
};

const LANES: usize = 4;
const STRIDE: usize = 16; // 4 × LANES for unrolled iteration
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
/// Caller must ensure AVX2 is enabled for the compilation target.
/// `p` through `end` must be valid for u64 reads.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn leading_scan<const FILL: u64>(
    mut p: *const u64,
    end: *const u64,
    base: *const u64,
    total: usize,
) -> *const u64 {
    // SAFETY: AVX2 is enabled for this compilation target. All pointer
    // arithmetic stays within bounds.
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
            let mut iters = (end as usize - p as usize) / (STRIDE * core::mem::size_of::<u64>());
            while iters > 0 {
                if FILL == 0 {
                    let d0 = _mm256_load_si256(p.cast::<__m256i>());
                    let d1 = _mm256_load_si256(p.add(LANES).cast::<__m256i>());
                    let d2 = _mm256_load_si256(p.add(LANES * 2).cast::<__m256i>());
                    let d3 = _mm256_load_si256(p.add(LANES * 3).cast::<__m256i>());
                    let any01 = _mm256_or_si256(d0, d1);
                    let any23 = _mm256_or_si256(d2, d3);
                    let any = _mm256_or_si256(any01, any23);
                    if _mm256_testz_si256(any, any) == 0 {
                        break;
                    }
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d0 = _mm256_load_si256(p.cast::<__m256i>());
                    let x0 = _mm256_xor_si256(d0, fill_vec);
                    let d1 = _mm256_load_si256(p.add(LANES).cast::<__m256i>());
                    let x1 = _mm256_xor_si256(d1, fill_vec);
                    let d2 = _mm256_load_si256(p.add(LANES * 2).cast::<__m256i>());
                    let x2 = _mm256_xor_si256(d2, fill_vec);
                    let d3 = _mm256_load_si256(p.add(LANES * 3).cast::<__m256i>());
                    let x3 = _mm256_xor_si256(d3, fill_vec);
                    let any01 = _mm256_or_si256(x0, x1);
                    let any23 = _mm256_or_si256(x2, x3);
                    let any = _mm256_or_si256(any01, any23);
                    if _mm256_testz_si256(any, any) == 0 {
                        break;
                    }
                }
                p = p.add(STRIDE);
                iters -= 1;
            }
        } else {
            // 4×-unrolled unaligned path.
            let mut iters = total / STRIDE;
            while iters > 0 {
                if FILL == 0 {
                    let d0 = _mm256_loadu_si256(p.cast::<__m256i>());
                    let d1 = _mm256_loadu_si256(p.add(LANES).cast::<__m256i>());
                    let d2 = _mm256_loadu_si256(p.add(LANES * 2).cast::<__m256i>());
                    let d3 = _mm256_loadu_si256(p.add(LANES * 3).cast::<__m256i>());
                    let any01 = _mm256_or_si256(d0, d1);
                    let any23 = _mm256_or_si256(d2, d3);
                    let any = _mm256_or_si256(any01, any23);
                    if _mm256_testz_si256(any, any) == 0 {
                        break;
                    }
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let d0 = _mm256_loadu_si256(p.cast::<__m256i>());
                    let x0 = _mm256_xor_si256(d0, fill_vec);
                    let d1 = _mm256_loadu_si256(p.add(LANES).cast::<__m256i>());
                    let x1 = _mm256_xor_si256(d1, fill_vec);
                    let d2 = _mm256_loadu_si256(p.add(LANES * 2).cast::<__m256i>());
                    let x2 = _mm256_xor_si256(d2, fill_vec);
                    let d3 = _mm256_loadu_si256(p.add(LANES * 3).cast::<__m256i>());
                    let x3 = _mm256_xor_si256(d3, fill_vec);
                    let any01 = _mm256_or_si256(x0, x1);
                    let any23 = _mm256_or_si256(x2, x3);
                    let any = _mm256_or_si256(any01, any23);
                    if _mm256_testz_si256(any, any) == 0 {
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
