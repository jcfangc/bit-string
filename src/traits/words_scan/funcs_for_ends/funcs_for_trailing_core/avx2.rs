// ═══════════════════════════════════════════════════════════════════════
// AVX2 backend — 256-bit / 4-lane reverse scan.
// ═══════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_loadu_si256, _mm256_or_si256, _mm256_set1_epi64x, _mm256_testz_si256,
    _mm256_xor_si256,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_loadu_si256, _mm256_or_si256, _mm256_set1_epi64x, _mm256_testz_si256,
    _mm256_xor_si256,
};

const LANES: usize = 4;
const STRIDE: usize = 16;

/// AVX2 reverse scan: scans backwards from `wi_end` and advances `done`
/// past all-FILL 256-bit chunks.
///
/// Returns the updated `done` count (total words consumed from right).
///
/// # Safety
///
/// Caller must ensure AVX2 is enabled for the compilation target.
/// `ptr` through `ptr.add(wi_end + 1)` must be valid for u64 reads.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn trailing_scan<const FILL: u64>(
    ptr: *const u64,
    wi_end: usize,
    mut done: usize,
    total_words: usize,
) -> usize {
    // SAFETY: AVX2 is enabled for this compilation target. All pointer
    // arithmetic stays within bounds.
    unsafe {
        // Four vectors are combined before testing. This keeps the
        // all-matching hot path to one branch per 16 words, matching
        // the forward scanner's throughput-oriented layout.
        while done + STRIDE <= total_words {
            let chunk_start = wi_end + 1 - (done + STRIDE);
            if FILL == 0 {
                let d0 = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                let d1 = _mm256_loadu_si256(ptr.add(chunk_start + LANES).cast::<__m256i>());
                let d2 = _mm256_loadu_si256(ptr.add(chunk_start + LANES * 2).cast::<__m256i>());
                let d3 = _mm256_loadu_si256(ptr.add(chunk_start + LANES * 3).cast::<__m256i>());
                let any01 = _mm256_or_si256(d0, d1);
                let any23 = _mm256_or_si256(d2, d3);
                let any = _mm256_or_si256(any01, any23);
                if _mm256_testz_si256(any, any) == 0 {
                    break;
                }
            } else {
                let fill_vec = _mm256_set1_epi64x(FILL as i64);
                let d0 = _mm256_loadu_si256(ptr.add(chunk_start).cast::<__m256i>());
                let x0 = _mm256_xor_si256(d0, fill_vec);
                let d1 = _mm256_loadu_si256(ptr.add(chunk_start + LANES).cast::<__m256i>());
                let x1 = _mm256_xor_si256(d1, fill_vec);
                let d2 = _mm256_loadu_si256(ptr.add(chunk_start + LANES * 2).cast::<__m256i>());
                let x2 = _mm256_xor_si256(d2, fill_vec);
                let d3 = _mm256_loadu_si256(ptr.add(chunk_start + LANES * 3).cast::<__m256i>());
                let x3 = _mm256_xor_si256(d3, fill_vec);
                let any01 = _mm256_or_si256(x0, x1);
                let any23 = _mm256_or_si256(x2, x3);
                let any = _mm256_or_si256(any01, any23);
                if _mm256_testz_si256(any, any) == 0 {
                    break;
                }
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
