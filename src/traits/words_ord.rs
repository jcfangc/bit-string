use core::cmp::Ordering;

/// Word-level lexicographic comparison on `[u64]` backing storage.
///
/// Compares words from index 0 upward (LSB-first bit order within each word).
/// Returns `Some(Ordering)` at the first differing word, or `None` when all
/// `count` words are identical.
///
/// `other` must be word-aligned.  `self` may have an intra-word `offset`
/// (0 = word-aligned), in which case each logical word is reconstructed as
/// a shifted window `(self[i] >> shift) | (self[i+1] << (64-shift))`.
///
/// Both paths dispatch to SIMD backends when available:
/// - AVX2 (x86/x86_64, 4×u64 per iteration)
/// - SSE2 (x86/x86_64, 2×u64 per iteration)
/// - NEON (aarch64, 2×u64 per iteration)
///
/// Short inputs fall back to scalar in all backends.
pub(crate) trait WordsOrd {
    /// `self` is pre-trimmed haystack `words[base..]`.
    /// `needle` is always word-aligned (pre-trimmed by the caller).
    /// `full_words` is the number of complete u64 words to compare.
    /// `haystack_shift` is the intra-word offset within the first word.
    /// When `HS_WORD_ALIGNED` is `true`, `haystack_shift == 0` is
    /// guaranteed and the aligned backend is used unconditionally.
    fn cmp_words<const HS_WORD_ALIGNED: bool>(
        &self,
        needle: &[u64],
        full_words: usize,
        haystack_shift: usize,
    ) -> Option<Ordering>;
}

pub(crate) mod funcs_for_cmp_aligned_core;
pub(crate) mod funcs_for_cmp_unaligned_core;
pub(crate) mod impls_for_u64_slice;
