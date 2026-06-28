use super::WordsFind;
use super::funcs_for_contains_core;
use super::funcs_for_find_core;
use super::funcs_for_rfind_core;

impl WordsFind for [u64] {
    #[inline]
    fn find_any_candidate<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        funcs_for_contains_core::find_any_candidate(
            self,
            haystack_bit_len,
            needle_words,
            needle_bit_len,
            verify,
        )
    }

    #[inline]
    fn find_first_word<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        funcs_for_find_core::find_first_word(
            self,
            haystack_bit_len,
            needle_words,
            needle_bit_len,
            verify,
        )
    }

    #[inline]
    fn find_last_word<F>(
        &self,
        haystack_bit_len: usize,
        needle_words: &[u64],
        needle_bit_len: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        funcs_for_rfind_core::find_last_word(
            self,
            haystack_bit_len,
            needle_words,
            needle_bit_len,
            verify,
        )
    }
}
