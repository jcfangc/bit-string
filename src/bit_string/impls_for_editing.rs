use super::*;

impl BitString {
    /// Truncate `self.words` to `words` elements, then lazily shrink the
    /// allocation when capacity exceeds `2 × words`.
    #[inline]
    fn truncate_words(&mut self, words: usize) {
        self.words.truncate(words);
        if self.words.capacity() > words * 2 {
            self.words.shrink_to(words);
        }
    }
}

mod impls_for_concat;
mod impls_for_drain;
mod impls_for_extend;
mod impls_for_insert_remove;
mod impls_for_push_pop;
mod impls_for_replace;
mod impls_for_retain;
mod impls_for_reverse_bits;
mod impls_for_set;
mod impls_for_slice;
mod impls_for_truncate;

#[cfg(test)]
mod tests_for_invariants;
