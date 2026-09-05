use super::{Oct, PackedSymbol, SparseByte, WideCode, oct, packed_as, wide};
use bit_string::PackedString;
use int_intervals::UsizeCO;

#[test]
fn packed_str_to_packed_string_preserves_views_and_owns_storage() {
    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    let empty = empty_owner.as_packed_str().to_packed_string();
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_materialized = oct_view.to_packed_string();
    assert_eq!(oct_materialized.char_len(), 4);
    assert_eq!(
        oct_materialized.to_vec(),
        oct_codes[20..24]
            .iter()
            .copied()
            .map(oct)
            .collect::<Vec<_>>()
    );
    assert!(oct_materialized.as_packed_str() == oct_view);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_materialized = wide_view.to_packed_string();
    assert!(wide_materialized.as_packed_str() == wide_view);
    assert_eq!(wide_materialized.bits().bit_len(), 35);

    let sparse_materialized = {
        let owner = packed_as::<SparseByte, 8>(&[255, 3, 0], |code| match code {
            0 => SparseByte::Zero,
            3 => SparseByte::Middle,
            255 => SparseByte::Maximum,
            _ => unreachable!(),
        });
        owner
            .as_packed_str()
            .slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .to_packed_string()
    };
    assert_eq!(
        sparse_materialized.to_vec(),
        vec![SparseByte::Middle, SparseByte::Zero]
    );
}

#[test]
fn packed_string_as_packed_str_covers_the_full_owner_without_copying_values() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    let empty_view = empty.as_packed_str();
    assert!(empty_view.is_empty());
    assert_eq!(empty_view.char_len(), empty.char_len());
    assert!(empty_view.to_packed_string().bits() == empty.bits());

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_view = binary.as_packed_str();
    assert_eq!(binary_view.iter().collect::<Vec<_>>(), binary.to_vec());
    assert!(binary_view.to_packed_string().bits() == binary.bits());

    let bytes = packed_as::<SparseByte, 8>(&[0, 255], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.char_len(), 2);
    assert_eq!(byte_view.iter().collect::<Vec<_>>(), bytes.to_vec());
    assert!(byte_view.to_packed_string().bits() == bytes.bits());

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_owner.as_packed_str();
    assert_eq!(
        oct_view.iter().collect::<Vec<_>>(),
        oct_codes.iter().copied().map(oct).collect::<Vec<_>>()
    );
    assert!(oct_view.to_packed_string().bits() == oct_owner.bits());

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_owner.as_packed_str();
    assert_eq!(
        wide_view.iter().collect::<Vec<_>>(),
        wide_codes.iter().copied().map(wide).collect::<Vec<_>>()
    );
    assert!(wide_view.to_packed_string().bits() == wide_owner.bits());
    assert!(wide_view == wide_view.clone());
}
