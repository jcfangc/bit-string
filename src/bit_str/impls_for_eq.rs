use crate::BitStr;

impl PartialEq for BitStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.bit_len != other.bit_len {
            return false;
        }
        if self.bit_len == 0 {
            return true;
        }
        self.bits_equal_at(0, *other)
    }
}

impl Eq for BitStr<'_> {}

#[cfg(test)]
mod tests_for_eq;
