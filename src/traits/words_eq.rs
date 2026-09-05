/// Word-level equality comparison on `[u64]` backing storage.
///
/// The caller pre-trims both slices to the relevant backing words and passes
/// the haystack's intra-word shift separately:
/// - `haystack_shift == 0` — word-aligned, uses [`funcs_for_eq_words_aligned_core`].
/// - `haystack_shift != 0` — unaligned shifted-window, uses [`funcs_for_eq_words_unaligned_core`].
///
/// This trait does not perform bit-level slicing or derive backing-word
/// offsets; those are handled by the higher-level view implementation.
///
/// Short inputs fall back to scalar before SIMD dispatch.
pub(crate) trait WordsEq {
    /// Compares `full_words` logical haystack words with `needle`.
    ///
    /// `self` is the pre-trimmed haystack backing slice.
    /// `needle` is word-aligned and pre-trimmed by the caller.
    /// `haystack_shift` is the physical start offset within `self[0]`.
    /// When `HS_WORD_ALIGNED` is `true`, `haystack_shift == 0` is
    /// guaranteed and the aligned backend is used unconditionally.
    /// When it is `false`, no alignment guarantee is made.
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
