/// Word-level equality comparison on `[u64]` backing storage.
///
/// Accepts a bit-level `offset` and handles word slicing and intra-word
/// shift internally:
/// - `offset % WORD_BITS == 0` — word-aligned, uses [`funcs_for_eq_words_aligned_core`].
/// - `offset % WORD_BITS != 0` — unaligned shifted-window, uses [`funcs_for_eq_words_unaligned_core`].
///
/// Short inputs fall back to scalar in both backends.
pub(crate) trait BitsEq {
    /// Returns `true` if `other` matches `self` starting at `offset` bits.
    ///
    /// `count` is the number of full `u64` words to compare (computed from
    /// the needle bit length).  The intra-word shift and word slicing are
    /// derived from `offset` internally.
    /// `self` is pre-trimmed haystack `words[base..]`.
    /// `needle` is always word-aligned (pre-trimmed by the caller).
    /// `full_words` is `needle_bit_len / WORD_BITS` — how many
    /// complete u64 words to compare.
    /// `haystack_shift` is `original_offset % WORD_BITS`.
    /// When `HS_WORD_ALIGNED` is `true`, `haystack_shift == 0` is
    /// guaranteed and the aligned backend is used unconditionally.
    fn eq_words<const HS_WORD_ALIGNED: bool>(
        &self,
        needle: &[u64],
        full_words: usize,
        haystack_shift: usize,
    ) -> bool;
}

pub(crate) mod funcs_for_eq_words_aligned_core;
pub(crate) mod funcs_for_eq_words_unaligned_core;
pub(crate) mod impls_for_u64_slice;
