use super::{PackedSymbol, SparseByte, Symbol};
use bit_string::{BitString, PackedString, traits::PackedChar};

#[test]
fn generated_sparse_code_round_trip_is_executable() {
    assert_eq!(SparseByte::Zero.code(), 0);
    assert_eq!(SparseByte::Middle.code(), 3);
    assert_eq!(SparseByte::Maximum.code(), u8::MAX);

    for code in 0..=u8::MAX {
        let expected = match code {
            0 => Some(SparseByte::Zero),
            3 => Some(SparseByte::Middle),
            u8::MAX => Some(SparseByte::Maximum),
            _ => None,
        };
        assert_eq!(SparseByte::from_code(code), expected);
    }
}

#[test]
fn packed_attribute_encoding_round_trips_through_packed_string() {
    let string = PackedString::<Symbol, 2>::from_chars([Symbol::Two, Symbol::Zero, Symbol::One]);
    assert_eq!(string.get(0), Some(Symbol::Two));
    assert_eq!(string.get(1), Some(Symbol::Zero));
    assert_eq!(string.get(2), Some(Symbol::One));
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ManualSymbol;

impl PackedChar<1> for ManualSymbol {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

#[test]
fn manual_packed_char_implementations_remain_supported() {
    assert_eq!(
        PackedString::<ManualSymbol, 1>::from_chars([ManualSymbol]).char_len(),
        1
    );
}

#[test]
fn attribute_macro_generates_packed_char_impl() {
    let string =
        PackedString::<PackedSymbol, 1>::from_chars([PackedSymbol::One, PackedSymbol::Zero]);
    assert_eq!(string.get(0), Some(PackedSymbol::One));
}

#[test]
fn from_bits_rejects_misaligned_and_unknown_codes() {
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true])).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true, true])).is_none());
}
