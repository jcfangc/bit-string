//! Packed-character tests grouped by API surface.

use bit_string::{PackedString, packed, traits::PackedChar};

#[packed(bits = 1)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PackedSymbol {
    Zero = 0,
    One = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[packed(bits = 2)]
pub(crate) enum Symbol {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[packed(bits = 3)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Oct {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[packed(bits = 8)]
pub(crate) enum SparseByte {
    Maximum = 255,
    Zero = 0,
    Middle = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WideCode(pub(crate) u8);

impl PackedChar<7> for WideCode {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code < 128).then_some(Self(code))
    }
}

pub(crate) fn symbol(code: u8) -> Symbol {
    match code {
        0 => Symbol::Zero,
        1 => Symbol::One,
        2 => Symbol::Two,
        _ => unreachable!(),
    }
}

pub(crate) fn oct(code: u8) -> Oct {
    match code {
        0 => Oct::V0,
        1 => Oct::V1,
        2 => Oct::V2,
        3 => Oct::V3,
        4 => Oct::V4,
        5 => Oct::V5,
        6 => Oct::V6,
        7 => Oct::V7,
        _ => unreachable!(),
    }
}

pub(crate) fn wide(code: u8) -> WideCode {
    WideCode(code)
}

pub(crate) fn packed(codes: &[u8]) -> PackedString<Symbol, 2> {
    PackedString::from_chars(codes.iter().copied().map(symbol))
}

pub(crate) fn packed_as<C, const BITS: u8>(
    codes: &[u8],
    decode: fn(u8) -> C,
) -> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    PackedString::from_chars(codes.iter().copied().map(decode))
}

#[path = "packed/tests_for_access.rs"]
mod tests_for_access;
#[path = "packed/tests_for_construction.rs"]
mod tests_for_construction;
#[path = "packed/tests_for_conversion.rs"]
mod tests_for_conversion;
#[path = "packed/tests_for_editing.rs"]
mod tests_for_editing;
#[path = "packed/tests_for_eq.rs"]
mod tests_for_eq;
#[path = "packed/tests_for_iter.rs"]
mod tests_for_iter;
#[path = "packed/tests_for_matching.rs"]
mod tests_for_matching;
#[path = "packed/tests_for_ord.rs"]
mod tests_for_ord;
