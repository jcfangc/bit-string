use super::*;

#[test]
fn packed_str_get_decodes_bounds_offsets_and_cross_word_codes() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().get(0), None);
    assert_eq!(empty.as_packed_str().get(usize::MAX), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().get(0), Some(PackedSymbol::Zero));
    assert_eq!(binary.as_packed_str().get(1), Some(PackedSymbol::One));
    assert_eq!(binary.as_packed_str().get(2), None);

    let bytes = packed_as::<SparseByte, 8>(&[0, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.get(0), Some(SparseByte::Zero));
    assert_eq!(byte_view.get(2), Some(SparseByte::Maximum));
    assert_eq!(byte_view.get(3), None);
    assert_eq!(byte_view.get(usize::MAX), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string.as_packed_str();
    for (index, &code) in oct_codes.iter().enumerate() {
        assert_eq!(oct_view.get(index), Some(oct(code)));
    }
    assert_eq!(oct_view.get(oct_codes.len()), None);
    let oct_slice = oct_view.slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    for (index, &code) in oct_codes[20..24].iter().enumerate() {
        assert_eq!(oct_slice.get(index), Some(oct(code)));
    }
    assert_eq!(oct_slice.get(4), None);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string.as_packed_str();
    for (index, &code) in wide_codes.iter().enumerate() {
        assert_eq!(wide_view.get(index), Some(wide(code)));
    }
    assert_eq!(wide_view.get(wide_codes.len()), None);
    let wide_slice = wide_view.slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    for (index, &code) in wide_codes[8..13].iter().enumerate() {
        assert_eq!(wide_slice.get(index), Some(wide(code)));
    }
    assert_eq!(wide_slice.get(usize::MAX), None);
}

#[test]
fn packed_string_get_isolates_codes_at_width_boundaries() {
    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.to_vec(),
        vec![
            PackedSymbol::Zero,
            PackedSymbol::One,
            PackedSymbol::Zero,
            PackedSymbol::One,
        ]
    );
    assert_eq!(binary.get(4), None);
    assert_eq!(binary.get(usize::MAX), None);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    for (index, code) in [0, 255, 3, 255].into_iter().enumerate() {
        assert_eq!(bytes.get(index).map(PackedChar::code), Some(code));
    }

    let oct_codes: Vec<_> = (0..22)
        .map(|index| if index == 21 { 1 } else { index as u8 % 8 })
        .collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    for (index, &code) in oct_codes.iter().enumerate() {
        assert_eq!(oct_string.get(index).map(PackedChar::code), Some(code));
    }
    assert_eq!(
        oct_string.get(20).map(PackedChar::code),
        Some(oct_codes[20])
    );
    assert_eq!(oct_string.get(21).map(PackedChar::code), Some(1));

    let wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                1
            } else {
                (index * 11) as u8 % 128
            }
        })
        .collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    for (index, &code) in wide_codes.iter().enumerate() {
        assert_eq!(wide_string.get(index).map(PackedChar::code), Some(code));
    }
    assert_eq!(
        wide_string.get(8).map(PackedChar::code),
        Some(wide_codes[8])
    );
    assert_eq!(wide_string.get(9).map(PackedChar::code), Some(1));
    assert_eq!(wide_string.get(10), None);
}

#[test]
fn packed_string_first_is_index_zero_and_empty_safe() {
    let new = PackedString::<Symbol, 2>::new();
    let default = PackedString::<Symbol, 2>::default();
    let from_empty = PackedString::<Symbol, 2>::from_chars(core::iter::empty());
    for string in [&new, &default, &from_empty] {
        assert_eq!(string.first(), None);
        assert_eq!(string.first(), string.get(0));
    }

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.first(), Some(PackedSymbol::Zero));
    assert_eq!(binary.first(), binary.get(0));

    let bytes = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_eq!(bytes.first(), Some(SparseByte::Maximum));

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let mut oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(oct_string.first(), Some(Oct::V0));
    assert_eq!(oct_string.first(), oct_string.to_vec().first().copied());

    oct_string.insert(0, Oct::V7);
    assert_eq!(oct_string.first(), Some(Oct::V7));
    assert_eq!(oct_string.remove(0), Oct::V7);
    assert_eq!(oct_string.first(), Some(Oct::V0));
    assert_eq!(oct_string.set(0, Oct::V6), Some(Oct::V0));
    assert_eq!(oct_string.first(), Some(Oct::V6));
    oct_string.clear();
    assert_eq!(oct_string.first(), None);
    oct_string.push(Oct::V3);
    assert_eq!(oct_string.first(), Some(Oct::V3));

    let wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 0 {
                127
            } else {
                (index * 11) as u8 % 128
            }
        })
        .collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(wide_string.first(), Some(WideCode(127)));
    assert_eq!(wide_string.first(), wide_string.get(0));
}

#[test]
fn packed_string_get_matches_character_oracle_and_bounds() {
    let codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let string = packed_as::<Oct, 3>(&codes, oct);
    for (index, &code) in codes.iter().enumerate() {
        assert_eq!(string.get(index), Some(oct(code)));
        assert_eq!(string.get(index), string.as_packed_str().get(index));
        assert_eq!(string.get(index), string.as_packed_str().iter().nth(index));
    }
    assert_eq!(string.get(codes.len()), None);
    assert_eq!(string.get(codes.len() + 1), None);
    assert_eq!(string.get(usize::MAX), None);

    let all_zero = packed_as::<Oct, 3>(&[0; 22], oct);
    assert_eq!(all_zero.get(0), Some(Oct::V0));
    assert_eq!(all_zero.get(21), Some(Oct::V0));

    let maximum = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_eq!(maximum.get(0), Some(SparseByte::Maximum));
    assert_eq!(maximum.get(1), Some(SparseByte::Zero));
    assert_eq!(maximum.get(2), None);
}

#[test]
fn packed_string_last_matches_the_final_character_and_pop() {
    let empty = PackedString::<Symbol, 2>::default();
    assert_eq!(empty.last(), None);
    assert_eq!(empty.last(), empty.as_packed_str().iter().next_back());

    let mut oct_string = packed_as::<Oct, 3>(&[Oct::V7.code(); 22], oct);
    assert_eq!(oct_string.last(), Some(Oct::V7));
    assert_eq!(oct_string.last(), oct_string.get(21));
    assert_eq!(
        oct_string.last(),
        oct_string.as_packed_str().iter().next_back()
    );

    assert_eq!(oct_string.set(21, Oct::V0), Some(Oct::V7));
    assert_eq!(oct_string.last(), Some(Oct::V0));
    assert_eq!(oct_string.pop(), Some(Oct::V0));
    assert_eq!(oct_string.last(), Some(Oct::V7));

    while oct_string.pop().is_some() {}
    assert_eq!(oct_string.char_len(), 0);
    assert_eq!(oct_string.last(), None);
    assert_eq!(oct_string.pop(), None);

    let wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                127
            } else {
                (index * 11) as u8 % 128
            }
        })
        .collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(wide_string.last(), Some(WideCode(127)));
    assert_eq!(wide_string.last(), wide_string.get(9));

    let bytes = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_eq!(bytes.last(), Some(SparseByte::Zero));
}

#[test]
fn packed_str_first_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().first(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().first(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.first(), Some(SparseByte::Maximum));
    assert_eq!(byte_view.slice_from(1).first(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_from(2).first(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(21, 2).unwrap());
    assert_eq!(oct_view.first(), Some(oct(5)));
    assert_eq!(oct_view.first(), oct_view.clone().get(0));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(9, 2).unwrap());
    assert_eq!(wide_view.first(), Some(wide(wide_codes[9])));
}

#[test]
fn packed_str_last_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().last(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().last(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.last(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_until(2).last(), Some(SparseByte::Middle));
    assert_eq!(byte_view.slice_until(0).last(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 2).unwrap());
    assert_eq!(oct_view.last(), Some(oct(5)));
    assert_eq!(oct_view.last(), oct_view.clone().get(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 2).unwrap());
    assert_eq!(wide_view.last(), Some(wide(wide_codes[9])));
    assert_eq!(wide_string.as_packed_str().slice_from(16).last(), None);
}

#[test]
fn packed_string_is_empty_tracks_zero_packed_bits() {
    let new = PackedString::<super::Symbol, 2>::new();
    let default = PackedString::<super::Symbol, 2>::default();
    let from_empty = PackedString::<super::Symbol, 2>::from_chars(core::iter::empty());
    for value in [&new, &default, &from_empty] {
        assert!(value.is_empty());
        assert_eq!(value.char_len(), 0);
        assert_eq!(value.bits().bit_len(), 0);
    }

    let one_zero = super::packed(&[0]);
    assert!(!one_zero.is_empty());
    assert_eq!(one_zero.char_len(), 1);
    assert_eq!(one_zero.bits().bit_len(), 2);

    let all_zero = super::packed(&[0; 64]);
    assert!(!all_zero.is_empty());
    assert_eq!(all_zero.char_len(), 64);
    assert_eq!(all_zero.bits().bit_len(), 128);

    let binary = packed_as::<PackedSymbol, 1>(&[0], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let oct = packed_as::<Oct, 3>(&[0; 22], super::oct);
    let wide = packed_as::<WideCode, 7>(&[0; 10], super::wide);
    let bytes = packed_as::<SparseByte, 8>(&[0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    assert!(!binary.is_empty());
    assert!(!oct.is_empty());
    assert!(!wide.is_empty());
    assert!(!bytes.is_empty());

    let mut popped = super::packed(&[0]);
    assert!(!popped.is_empty());
    assert_eq!(popped.pop(), Some(super::Symbol::Zero));
    assert!(popped.is_empty());

    let mut cleared = super::packed(&[0, 1, 2]);
    cleared.clear();
    assert!(cleared.is_empty());

    let mut truncated = super::packed(&[0, 1, 2]);
    truncated.truncate(0);
    assert!(truncated.is_empty());

    let mut drained = super::packed(&[0, 1, 2]);
    drained.drain_interval_assign(UsizeCO::checked_from_start_len(0, 3).unwrap());
    assert!(drained.is_empty());
}
