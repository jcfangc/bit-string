use super::{Oct, PackedSymbol, SparseByte, Symbol, WideCode, oct, packed_as, symbol, wide};
use bit_string::{BitString, PackedString, traits::PackedChar};

fn assert_packed_owner<C, const BITS: u8>(codes: &[u8], decode: fn(u8) -> C)
where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let owner = packed_as(codes, decode);
    let expected = codes.iter().copied().map(decode).collect::<Vec<_>>();
    assert_eq!(owner.char_len(), codes.len());
    assert_eq!(owner.bits_per_char(), usize::from(BITS));
    assert_eq!(owner.bits().bit_len(), codes.len() * usize::from(BITS));
    assert_eq!(owner.to_vec(), expected);
    for (index, character) in expected.iter().copied().enumerate() {
        assert_eq!(owner.get(index), Some(character));
    }
    assert_eq!(owner.get(codes.len()), None);
    assert!(owner.as_packed_str().iter().collect::<Vec<_>>() == expected);
    assert!(owner.clone().into_bits() == owner.bits().clone());
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ManualSymbol;

#[derive(Clone, Copy, PartialEq, Eq)]
struct UnsupportedWidth;

impl PackedChar<0> for UnsupportedWidth {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

impl PackedChar<9> for UnsupportedWidth {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

impl PackedChar<1> for ManualSymbol {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

#[path = "tests_for_construction/tests_for_derive_repr.rs"]
mod tests_for_derive_repr;

#[path = "tests_for_construction/tests_for_input_construction.rs"]
mod tests_for_input_construction;

#[path = "tests_for_construction/tests_for_width_layout.rs"]
mod tests_for_width_layout;

#[path = "tests_for_construction/tests_for_raw_bits_validation.rs"]
mod tests_for_raw_bits_validation;
