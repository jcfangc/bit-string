use super::*;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    #[inline]
    pub fn any(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn all(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn is_all_zeros(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn is_all_ones(&self) -> bool {
        todo!()
    }
}
