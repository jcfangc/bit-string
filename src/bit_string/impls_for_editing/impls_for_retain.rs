use crate::bit_string::bits::*;

use super::*;

impl BitString {
    /// Retains only the bits for which `f(bit)` returns `true`.
    ///
    /// Bits are processed in order and written back contiguously, preserving
    /// the relative order of retained bits.  Operates bit-by-bit.
    ///
    /// `f` may be [`FnMut`] — the predicate is allowed to carry mutable state.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        let mut write = 0usize;

        for read in 0..self.bit_len {
            let value = self.words.read_bit_at(read);

            if f(value) {
                self.words.set_bit_at(write, value);
                write += 1;
            }
        }

        self.truncate(write);
    }
}

#[cfg(test)]
mod tests_for_retain;
