use crate::WORD_BITS;
use crate::traits::*;

use super::*;

impl BitString {
    /// Retains only the bits for which `f(bit)` returns `true`.
    ///
    /// Bits are processed in order and written back contiguously, preserving
    /// the relative order of retained bits.
    ///
    /// `f` may be [`FnMut`] — the predicate is allowed to carry mutable state.
    /// If `f` panics, already processed words may have been compacted, but the
    /// bit string remains valid.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        let original_len = self.bit_len;
        let mut read = 0usize;
        let mut write = 0usize;

        while read < original_len {
            let batch_len = (original_len - read).min(WORD_BITS);
            let source = self.words.read_word_at::<false>(read);
            let mut retained = 0u64;
            let mut retained_len = 0usize;

            for bit_index in 0..batch_len {
                let value = source & (1u64 << bit_index) != 0;
                if f(value) {
                    if value {
                        retained |= 1u64 << retained_len;
                    }
                    retained_len += 1;
                }
            }

            if retained_len > 0 && (write != read || retained_len != batch_len) {
                self.words.clear_bits_at(write, retained_len);
                self.words
                    .write_word_at::<false>(write, retained, retained_len);
            }

            write += retained_len;
            read += batch_len;
        }

        self.truncate(write);
    }
}

#[cfg(test)]
mod tests_for_retain;
