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

    #[inline]
    pub fn slice_from(&self, _at: usize) -> Self {
        todo!()
    }

    #[inline]
    pub fn slice_until(&self, _to: usize) -> Self {
        todo!()
    }
}
