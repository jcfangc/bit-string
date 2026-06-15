use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        let mut write = 0usize;

        for read in 0..self.len {
            let value = Bits::bit_at(&self.bits, read);

            if f(value) {
                Bits::set_bit(&mut self.bits, write, value);
                write += 1;
            }
        }

        self.truncate(write);
    }
}

#[cfg(test)]
mod tests_for_retain;
