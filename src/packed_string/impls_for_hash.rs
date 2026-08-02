use core::hash::{Hash, Hasher};

use super::*;

impl<C: PackedChar> Hash for PackedString<C> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!("PackedString::hash")
    }
}
