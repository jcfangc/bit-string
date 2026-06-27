use super::*;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.count_ones() != 0
    }

    #[inline]
    pub fn all(&self) -> bool {
        self.count_ones() == self.bit_len
    }

    #[inline]
    pub fn is_all_zeros(&self) -> bool {
        !self.any()
    }

    #[inline]
    pub fn is_all_ones(&self) -> bool {
        self.all()
    }
}
