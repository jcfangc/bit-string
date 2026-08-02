use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn matches_at(&self, _index: usize, _pattern: &Self) -> bool {
        unimplemented!("PackedString::matches_at")
    }

    pub fn starts_with(&self, _prefix: &Self) -> bool {
        unimplemented!("PackedString::starts_with")
    }

    pub fn ends_with(&self, _suffix: &Self) -> bool {
        unimplemented!("PackedString::ends_with")
    }

    pub fn contains(&self, _needle: &Self) -> bool {
        unimplemented!("PackedString::contains")
    }

    pub fn find(&self, _needle: &Self) -> Option<usize> {
        unimplemented!("PackedString::find")
    }

    pub fn rfind(&self, _needle: &Self) -> Option<usize> {
        unimplemented!("PackedString::rfind")
    }

    pub fn strip_prefix(&self, _prefix: &Self) -> Option<Self> {
        unimplemented!("PackedString::strip_prefix")
    }

    pub fn strip_suffix(&self, _suffix: &Self) -> Option<Self> {
        unimplemented!("PackedString::strip_suffix")
    }
}
