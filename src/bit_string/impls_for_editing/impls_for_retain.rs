use crate::bit_string::bits::*;

use super::*;

impl BitString {
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
