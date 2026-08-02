use super::*;

impl<'ps, C: PackedChar> PackedStr<'ps, C> {
    pub fn source(&self) -> &'ps PackedString<C> {
        unimplemented!("PackedStr::source")
    }

    pub fn char_start(&self) -> usize {
        unimplemented!("PackedStr::char_start")
    }

    pub fn char_len(&self) -> usize {
        unimplemented!("PackedStr::char_len")
    }

    pub fn is_empty(&self) -> bool {
        unimplemented!("PackedStr::is_empty")
    }

    pub fn get(&self, _index: usize) -> Option<C> {
        unimplemented!("PackedStr::get")
    }

    pub fn first(&self) -> Option<C> {
        unimplemented!("PackedStr::first")
    }

    pub fn last(&self) -> Option<C> {
        unimplemented!("PackedStr::last")
    }
}
