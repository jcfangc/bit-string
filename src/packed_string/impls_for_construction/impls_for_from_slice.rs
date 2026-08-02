use super::*;

impl<C: PackedChar> From<&[C]> for PackedString<C> {
    fn from(_characters: &[C]) -> Self {
        unimplemented!("PackedString::from(&[C])")
    }
}

impl<C: PackedChar, const N: usize> From<[C; N]> for PackedString<C> {
    fn from(_characters: [C; N]) -> Self {
        unimplemented!("PackedString::from([C; N])")
    }
}
