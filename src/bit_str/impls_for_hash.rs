use core::hash::{Hash, Hasher};

use crate::WORD_BITS;

use crate::BitStr;

mod inner;

impl Hash for BitStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.start % WORD_BITS == 0 {
            self.hash_inner::<true, H>(state)
        } else {
            self.hash_inner::<false, H>(state)
        }
    }
}

#[cfg(test)]
mod tests_for_hash;
