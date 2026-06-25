use core::hash::{Hash, Hasher};

use crate::BitString;

impl Hash for BitString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bit_str().hash(state);
    }
}
