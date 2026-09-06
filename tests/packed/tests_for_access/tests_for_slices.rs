use super::*;

#[test]
fn packed_str_slice_uses_clamped_character_ranges() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    let full = view.slice(UsizeCO::checked_from_start_len(0, 5).unwrap());
    assert_eq!(full.iter().collect::<Vec<_>>(), owner.to_vec());

    let middle = view.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert_eq!(
        middle.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One, super::Symbol::Two, super::Symbol::One]
    );
    let singleton = view.slice(UsizeCO::checked_from_start_len(4, 1).unwrap());
    assert_eq!(
        singleton.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    let clamped_end = view.slice(UsizeCO::checked_from_start_len(3, 99).unwrap());
    assert_eq!(
        clamped_end.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One, super::Symbol::Zero]
    );
    assert!(
        view.slice(UsizeCO::checked_from_start_len(99, 1).unwrap())
            .is_empty()
    );
    assert!(
        view.slice(UsizeCO::checked_from_start_len(5, 1).unwrap())
            .is_empty()
    );

    let nested = middle.slice(UsizeCO::checked_from_start_len(1, 1).unwrap());
    assert_eq!(nested.iter().collect::<Vec<_>>(), vec![super::Symbol::Two]);
    assert_eq!(nested.to_packed_string().to_vec(), vec![super::Symbol::Two]);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_slice = binary
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        binary_slice.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::One, PackedSymbol::One]
    );

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_slice = bytes
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        byte_slice.to_packed_string().to_vec(),
        vec![SparseByte::Maximum, SparseByte::Middle]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_slice = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        oct_slice
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
    assert_eq!(oct_slice.to_packed_string().bits().bit_len(), 6);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_slice = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap())
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert_eq!(
        wide_slice
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![99, 110, 121]
    );
    assert_eq!(wide_slice.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}

#[test]
fn packed_str_slice_from_clamps_at_the_view_end() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    assert_eq!(
        view.slice_from(0).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );
    assert_eq!(
        view.slice_from(1).iter().collect::<Vec<_>>(),
        vec![
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
            super::Symbol::Zero
        ]
    );
    assert_eq!(
        view.slice_from(4).iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    assert!(view.slice_from(5).is_empty());
    assert!(view.slice_from(6).is_empty());
    assert!(view.slice_from(usize::MAX).is_empty());

    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    assert!(empty_owner.as_packed_str().slice_from(0).is_empty());
    assert!(
        empty_owner
            .as_packed_str()
            .slice_from(usize::MAX)
            .is_empty()
    );

    let nested = view
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap())
        .slice_from(1);
    assert_eq!(
        nested.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Two, super::Symbol::One]
    );

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_suffix = binary.as_packed_str().slice_from(1);
    assert_eq!(
        binary_suffix.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::One, PackedSymbol::One]
    );
    assert_eq!(binary_suffix.to_packed_string().bits().bit_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_suffix = bytes.as_packed_str().slice_from(1);
    assert_eq!(
        byte_suffix.iter().collect::<Vec<_>>(),
        vec![SparseByte::Maximum, SparseByte::Middle]
    );
    assert_eq!(byte_suffix.to_packed_string().bits().bit_len(), 16);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_suffix = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice_from(1);
    assert_eq!(
        oct_suffix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![5, 6, 7]
    );
    assert_eq!(oct_suffix.to_packed_string().bits().bit_len(), 9);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_suffix = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap())
        .slice_from(1);
    assert_eq!(
        wide_suffix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![99, 110, 121]
    );
    assert_eq!(wide_suffix.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}

#[test]
fn packed_str_slice_until_clamps_at_the_view_start() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    assert!(view.slice_until(0).is_empty());
    assert_eq!(
        view.slice_until(1).iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    assert_eq!(
        view.slice_until(4).iter().collect::<Vec<_>>(),
        vec![
            super::Symbol::Zero,
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
        ]
    );
    assert_eq!(
        view.slice_until(5).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );
    assert_eq!(
        view.slice_until(usize::MAX).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );

    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    assert!(empty_owner.as_packed_str().slice_until(0).is_empty());
    assert!(
        empty_owner
            .as_packed_str()
            .slice_until(usize::MAX)
            .is_empty()
    );

    let offset_owner = super::packed(&[0, 1, 2, 1, 0]);
    let offset_view = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert!(offset_view.slice_until(0).is_empty());
    assert_eq!(
        offset_view.slice_until(1).iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    let nested = offset_view.slice_until(2).slice_until(1);
    assert_eq!(nested.iter().collect::<Vec<_>>(), vec![super::Symbol::One]);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_prefix = binary.as_packed_str().slice_until(2);
    assert_eq!(
        binary_prefix.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::Zero, PackedSymbol::One]
    );
    assert_eq!(binary_prefix.to_packed_string().bits().bit_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_prefix = bytes.as_packed_str().slice_until(2);
    assert_eq!(
        byte_prefix.to_packed_string().to_vec(),
        vec![SparseByte::Zero, SparseByte::Maximum]
    );
    assert_eq!(byte_prefix.to_packed_string().bits().bit_len(), 16);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_prefix = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice_until(3);
    assert_eq!(
        oct_prefix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![4, 5, 6]
    );
    assert_eq!(oct_prefix.to_packed_string().bits().bit_len(), 9);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_prefix = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap())
        .slice_until(3);
    assert_eq!(
        wide_prefix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![88, 99, 110]
    );
    assert_eq!(wide_prefix.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}
