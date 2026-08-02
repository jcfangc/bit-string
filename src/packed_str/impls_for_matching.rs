use super::*;

impl<C, const BITS: u8> PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn matches_at(&self, _index: usize, _pattern: Self) -> bool {
        unimplemented!("PackedStr::matches_at")
    }

    pub fn starts_with(&self, _prefix: Self) -> bool {
        unimplemented!("PackedStr::starts_with")
    }

    pub fn ends_with(&self, _suffix: Self) -> bool {
        unimplemented!("PackedStr::ends_with")
    }

    pub fn contains(&self, _needle: Self) -> bool {
        unimplemented!("PackedStr::contains")
    }

    pub fn find(&self, _needle: Self) -> Option<usize> {
        unimplemented!("PackedStr::find")
    }

    pub fn rfind(&self, _needle: Self) -> Option<usize> {
        unimplemented!("PackedStr::rfind")
    }

    pub fn strip_prefix(&self, _prefix: Self) -> Option<Self> {
        unimplemented!("PackedStr::strip_prefix")
    }

    pub fn strip_suffix(&self, _suffix: Self) -> Option<Self> {
        unimplemented!("PackedStr::strip_suffix")
    }
}
