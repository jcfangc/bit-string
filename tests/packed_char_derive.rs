use bit_string::{PackedString, traits::PackedChar};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PackedChar)]
#[repr(u8)]
#[packed(bits = 2)]
enum Symbol {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[test]
fn derived_encoding_round_trips_through_packed_string() {
    let string = PackedString::<Symbol, 2>::from_chars([Symbol::Two, Symbol::Zero, Symbol::One]);
    assert_eq!(string.get(0), Some(Symbol::Two));
    assert_eq!(string.get(1), Some(Symbol::Zero));
    assert_eq!(string.get(2), Some(Symbol::One));
}
