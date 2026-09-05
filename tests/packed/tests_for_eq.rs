use super::{Oct, PackedSymbol, WideCode, oct, packed_as, wide};
use bit_string::PackedString;
use int_intervals::UsizeCO;

#[test]
fn packed_str_equality_compares_viewed_bits_not_source_identity() {
    let empty_left = PackedString::<PackedSymbol, 1>::new();
    let empty_right = PackedString::<PackedSymbol, 1>::new();
    assert!(empty_left.as_packed_str() == empty_right.as_packed_str());

    let binary_left = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_right = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(binary_left.as_packed_str() == binary_right.as_packed_str());
    assert!(binary_left.as_packed_str() != binary_left.as_packed_str().slice_from(1));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_equal_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_equal_view = oct_equal_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_view == oct_equal_view);
    assert!(oct_view == oct_view.clone());
    assert!(oct_owner.as_packed_str().slice_until(0) == oct_owner.as_packed_str().slice_from(24));
    assert!(
        oct_view
            != oct_owner
                .as_packed_str()
                .slice(UsizeCO::checked_from_start_len(20, 3).unwrap())
    );

    let mut oct_different_codes = oct_codes.clone();
    oct_different_codes[21] = 6;
    let oct_different_owner = packed_as::<Oct, 3>(&oct_different_codes, oct);
    let oct_different_view = oct_different_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_view != oct_different_view);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_equal_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_equal_view = wide_equal_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    assert!(wide_view == wide_equal_view);

    let mut wide_different_codes = wide_codes.clone();
    wide_different_codes[9] ^= 1;
    let wide_different_owner = packed_as::<WideCode, 7>(&wide_different_codes, wide);
    let wide_different_view = wide_different_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    assert!(wide_view != wide_different_view);
}
