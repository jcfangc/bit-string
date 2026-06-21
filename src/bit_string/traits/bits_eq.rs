/// Word-level equality comparison on `[u64]` backing storage.
///
/// A single method dispatches to the appropriate SIMD backend based on
/// the intra-word shift:
/// - `shift == 0` — word-aligned, uses [`funcs_for_eq_words_aligned_core`].
/// - `shift != 0` — unaligned shifted-window, uses [`funcs_for_eq_words_unaligned_core`].
///
/// Short inputs fall back to scalar in both backends.
pub(crate) trait BitsEq {
    /// Returns `true` if the first `count` words of `self` match `other`,
    /// compensating for an intra-word shift of `shift` bits.
    ///
    /// When `shift == 0` this is a direct word comparison.  When `shift != 0`
    /// each 64-bit window is computed as:
    ///
    /// ```text
    /// window[i] = (self[i] >> shift) | (self[i + 1] << (WORD_BITS - shift))
    /// ```
    fn eq_words(&self, other: &[u64], count: usize, shift: usize) -> bool;
}

pub(crate) mod funcs_for_eq_words_aligned_core;
pub(crate) mod funcs_for_eq_words_unaligned_core;
pub(crate) mod impls_for_u64_slice;
