use super::*;

impl BitString {
    #[inline]
    pub fn contains(&self, needle: crate::BitStr<'_>) -> bool {
        self.as_bit_str().contains(needle)
    }

    #[inline]
    pub fn find(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().find(needle)
    }

    #[inline]
    pub fn rfind(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().rfind(needle)
    }
}
