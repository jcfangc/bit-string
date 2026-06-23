use int_interval::UsizeCO;

use super::*;

impl<'bs> BitStr<'bs> {
    /// Returns a sub-view of the bits in `interval`.
    ///
    /// The interval is clamped to `[0, self.bit_len()]`. An interval beyond the
    /// view returns an empty result.
    #[inline]
    pub fn slice(&self, interval: UsizeCO) -> Self {
        let s = interval.start().min(self.bit_len);
        let e = interval.end_excl().min(self.bit_len).max(s);
        Self {
            source: self.source,
            start: self.start + s,
            bit_len: e - s,
        }
    }

    /// Returns a sub-view from `at` to the end.
    ///
    /// The start is clamped to `self.bit_len()`. When `at >= self.bit_len()`
    /// the result is empty.
    #[inline]
    pub fn slice_from(&self, at: usize) -> Self {
        let s = at.min(self.bit_len);
        Self {
            source: self.source,
            start: self.start + s,
            bit_len: self.bit_len - s,
        }
    }

    /// Returns a sub-view from the start to `to`.
    ///
    /// `to` is clamped to `self.bit_len()`. When `to == 0` the result is empty.
    #[inline]
    pub fn slice_until(&self, to: usize) -> Self {
        let e = to.min(self.bit_len);
        Self {
            source: self.source,
            start: self.start,
            bit_len: e,
        }
    }
}

#[cfg(test)]
mod tests_for_slice;
