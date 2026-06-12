use super::*;

impl BitString {
    #[inline]
    pub fn count_ones(&self) -> usize {
        funcs_for_count_ones::count_ones(&self.bits, self.len)
    }

    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.len - self.count_ones()
    }
}

mod funcs_for_count_ones;

#[cfg(test)]
mod tests_for_count_ones;
