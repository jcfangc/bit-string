/// SIMD-accelerated pattern-search operations on `[u64]` backing storage.
///
/// All positions returned are relative to the start of `self`. Callers
/// with an offset view (e.g. [`BitStr`](crate::BitStr)) must shift
/// positions accordingly.
pub(crate) trait BitsFind {
    /// Returns `Some(pos)` if any 64-bit window in the haystack matches
    /// the first word of `needle_words` AND `verify(pos)` succeeds.
    ///
    /// Uses **shift-outer, word-inner** ordering — does **not**
    /// guarantee the returned position is the earliest match.
    fn find_any_candidate<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool;

    /// Returns `Some(pos)` for the **earliest** position where the
    /// first word of `needle_words` matches AND `verify(pos)` succeeds.
    ///
    /// Uses **word-outer, shift-inner** ordering so positions are
    /// visited in increasing order.
    fn find_first_word<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool;

    /// Returns `Some(pos)` for the **rightmost** position where the
    /// first word of `needle_words` matches AND `verify(pos)` succeeds.
    ///
    /// Uses **word-outer reverse, shift-inner reverse** ordering so
    /// positions are visited in decreasing order.
    fn find_last_word<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool;
}

pub(crate) mod funcs_for_contains_core;
pub(crate) mod funcs_for_find_core;
pub(crate) mod funcs_for_rfind_core;
pub(crate) mod impls_for_u64_slice;
