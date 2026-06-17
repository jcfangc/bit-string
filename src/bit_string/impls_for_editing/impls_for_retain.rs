use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        let mut write = 0usize;

        for read in 0..self.bit_len {
            let value = Bits::read_a_bit_at(&self.words, read);

            if f(value) {
                Bits::set_a_bit_at(&mut self.words, write, value);
                write += 1;
            }
        }

        self.truncate(write);
    }
}

#[cfg(test)]
mod tests_for_retain;
