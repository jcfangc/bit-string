use super::{Oct, PackedString, Symbol, WideCode, oct, packed_as, wide};
use bit_string::traits::PackedChar;
use core::cmp::Ordering;
use int_intervals::UsizeCO;

fn assert_code_order<C, const BITS: u8>(
    left: &[u8],
    right: &[u8],
    decode: fn(u8) -> C,
    expected: Ordering,
) where
    C: PackedChar<BITS>,
{
    let left = packed_as(left, decode);
    let right = packed_as(right, decode);
    assert_eq!(left.cmp_string(&right), expected);
    assert_eq!(left.cmp(&right), expected);
    assert_eq!(left.as_packed_str().cmp(&right.as_packed_str()), expected);
}

#[test]
fn ordering_uses_packed_code_values() {
    let one = PackedString::<Symbol, 2>::from_chars([Symbol::One]);
    let two = PackedString::<Symbol, 2>::from_chars([Symbol::Two]);
    assert!(one < two);
    assert!(one.as_packed_str() < two.as_packed_str());
}

#[test]
fn packed_order_compares_numeric_codes_before_length() {
    assert_code_order::<Symbol, 2>(&[1], &[2], super::symbol, Ordering::Less);
    assert_code_order::<Symbol, 2>(&[2], &[1], super::symbol, Ordering::Greater);
    assert_code_order::<Symbol, 2>(&[2], &[2, 0], super::symbol, Ordering::Less);
    assert_code_order::<Symbol, 2>(&[0, 2, 0], &[0, 1, 2], super::symbol, Ordering::Greater);
    assert_code_order::<Symbol, 2>(&[1, 0], &[1, 2], super::symbol, Ordering::Less);
    assert_code_order::<Symbol, 2>(&[1, 2], &[1, 2], super::symbol, Ordering::Equal);

    assert_code_order::<super::PackedSymbol, 1>(
        &[0, 1],
        &[1, 0],
        |code| match code {
            0 => super::PackedSymbol::Zero,
            1 => super::PackedSymbol::One,
            _ => unreachable!(),
        },
        Ordering::Less,
    );
    assert_code_order::<super::SparseByte, 8>(
        &[0],
        &[255],
        |code| match code {
            0 => super::SparseByte::Zero,
            255 => super::SparseByte::Maximum,
            _ => unreachable!(),
        },
        Ordering::Less,
    );

    let oct_left = vec![0; 22];
    let mut oct_right = oct_left.clone();
    oct_right[21] = 7;
    assert_code_order::<Oct, 3>(&oct_left, &oct_right, oct, Ordering::Less);

    let wide_left = vec![0; 10];
    let mut wide_right = wide_left.clone();
    wide_right[9] = 127;
    assert_code_order::<WideCode, 7>(&wide_left, &wide_right, wide, Ordering::Less);

    let owner = PackedString::<Symbol, 2>::from_chars([
        Symbol::Zero,
        Symbol::One,
        Symbol::Two,
        Symbol::One,
    ]);
    let other =
        PackedString::<Symbol, 2>::from_chars([Symbol::Two, Symbol::One, Symbol::Two, Symbol::One]);
    let owner_view = owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    let other_view = other
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(owner_view.cmp(&other_view), Ordering::Equal);
    assert_eq!(owner_view.char_len(), 2);
    assert_eq!(other_view.char_len(), 2);
}
