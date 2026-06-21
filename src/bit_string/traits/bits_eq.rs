/// Word-level equality comparison on `[u64]` backing storage.
///
/// Two operations are provided:
/// - [`eq_words`] — direct word comparison for word-aligned sequences.
/// - [`eq_words_shifted`] — shifted-window comparison for unaligned sequences.
///
/// Each dispatches to the best available SIMD backend (AVX2, SSE2, NEON)
/// at compile time, falling back to scalar for small inputs.
pub(crate) trait BitsEq {
    /// Returns `true` if the first `count` words of `self` match `other`.
    ///
    /// Both slices must be word-aligned (i.e. the comparison starts at bit 0
    /// of each word). For fewer than [`SMALL_WORDS`](crate::SMALL_WORDS) words
    /// the scalar loop is used directly.
    fn eq_words(&self, other: &[u64], count: usize) -> bool;

    /// Returns `true` if the `count` shifted 64-bit windows of `self` match
    /// `other`, where each window is computed as:
    ///
    /// ```text
    /// window[i] = (self[i] >> shift) | (self[i + 1] << (WORD_BITS - shift))
    /// ```
    ///
    /// When `shift == 0` this delegates to [`eq_words`].  Short inputs fall
    /// back to scalar.
    fn eq_words_shifted(&self, other: &[u64], count: usize, shift: usize) -> bool;
}

pub(crate) mod funcs_for_eq_words_core;
pub(crate) mod funcs_for_eq_words_shifted_core;
pub(crate) mod impls_for_u64_slice;
