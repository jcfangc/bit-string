// ═══════════════════════════════════════════════════════════════════════
// SSE2 backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

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
    // SAFETY: SSE2 is enabled for this compilation target.
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
