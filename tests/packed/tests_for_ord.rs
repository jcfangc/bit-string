use super::{PackedString, Symbol};

#[test]
fn ordering_uses_packed_code_values() {
    let one = PackedString::<Symbol, 2>::from_chars([Symbol::One]);
    let two = PackedString::<Symbol, 2>::from_chars([Symbol::Two]);
    assert!(one < two);
    assert!(one.as_packed_str() < two.as_packed_str());
}
