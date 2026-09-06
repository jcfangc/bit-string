use super::{Oct, PackedString, UsizeCO, WideCode, oct, packed, packed_as, wide};
use bit_string::traits::PackedChar;

#[test]
fn packed_string_strip_prefix_returns_the_remaining_owner() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    let prefix = packed(&[0, 1]);
    assert_eq!(
        receiver.strip_prefix(&prefix).unwrap().to_vec(),
        vec![super::Symbol::Two, super::Symbol::One, super::Symbol::Zero]
    );
    assert_eq!(
        receiver.strip_prefix(&PackedString::<super::Symbol, 2>::new()),
        Some(receiver.clone())
    );
    assert!(receiver.strip_prefix(&receiver).unwrap().is_empty());
    assert!(receiver.strip_prefix(&packed(&[2, 1])).is_none());
    assert!(
        receiver
            .strip_prefix(&packed(&[0, 1, 2, 1, 0, 1]))
            .is_none()
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_prefix = packed_as::<Oct, 3>(&oct_codes[..20], oct);
    let oct_stripped = oct_receiver.strip_prefix(&oct_prefix).unwrap();
    assert_eq!(
        oct_stripped.to_vec(),
        oct_codes[20..].iter().copied().map(oct).collect::<Vec<_>>()
    );
    assert_eq!(oct_stripped.bits().bit_len(), 4 * 3);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_prefix = packed_as::<WideCode, 7>(&wide_codes[..8], wide);
    let wide_stripped = wide_receiver.strip_prefix(&wide_prefix).unwrap();
    assert_eq!(
        wide_stripped.to_vec(),
        wide_codes[8..]
            .iter()
            .copied()
            .map(wide)
            .collect::<Vec<_>>()
    );
    assert_eq!(wide_stripped.bits().bit_len(), 8 * 7);
}

#[test]
fn packed_str_strip_prefix_returns_the_remaining_character_view() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let prefix = packed(&[0, 1]);
    let stripped = receiver.strip_prefix(prefix.as_packed_str()).unwrap();
    assert_eq!(
        stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Two, super::Symbol::One, super::Symbol::Zero]
    );
    assert_eq!(stripped.char_len(), 3);
    assert_eq!(
        stripped.to_packed_string().to_vec(),
        vec![super::Symbol::Two, super::Symbol::One, super::Symbol::Zero]
    );

    let empty_prefix_owner = packed(&[]);
    let empty_prefix = empty_prefix_owner.as_packed_str();
    assert!(receiver.strip_prefix(empty_prefix).unwrap() == receiver);
    assert!(receiver.strip_prefix(receiver).unwrap().is_empty());
    assert!(empty_prefix.strip_prefix(empty_prefix).unwrap().is_empty());

    let different_first = packed(&[1, 1]);
    let different_middle = packed(&[0, 2]);
    let different_last = packed(&[0, 0]);
    let oversized = packed(&[0, 1, 2, 1, 0, 2]);
    assert!(
        receiver
            .strip_prefix(different_first.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_prefix(different_middle.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_prefix(different_last.as_packed_str())
            .is_none()
    );
    assert!(receiver.strip_prefix(oversized.as_packed_str()).is_none());
    assert!(
        empty_prefix
            .strip_prefix(packed(&[0]).as_packed_str())
            .is_none()
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_receiver = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_prefix = packed(&[1, 2]);
    let offset_stripped = offset_receiver
        .strip_prefix(offset_prefix.as_packed_str())
        .unwrap();
    assert_eq!(
        offset_stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    assert!(
        offset_receiver
            .strip_prefix(offset_receiver.slice_from(offset_receiver.char_len()))
            .unwrap()
            == offset_receiver
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary
            .as_packed_str()
            .strip_prefix(binary.as_packed_str().slice_until(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::PackedSymbol::One, super::PackedSymbol::One]
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes
            .as_packed_str()
            .strip_prefix(bytes.as_packed_str().slice_until(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::SparseByte::Middle, super::SparseByte::Zero]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_stripped = oct_receiver
        .strip_prefix(oct_receiver.slice_until(2))
        .unwrap();
    assert_eq!(
        oct_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![6, 7]
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_stripped = wide_receiver
        .strip_prefix(wide_receiver.slice_until(2))
        .unwrap();
    assert_eq!(
        wide_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![110, 121]
    );

    assert_eq!(offset_receiver.char_len(), 3);
    assert_eq!(offset_receiver.get(0), Some(super::Symbol::One));
    assert_eq!(offset_receiver.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_string_strip_suffix_returns_the_remaining_owner() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    let suffix = packed(&[1, 0]);
    assert_eq!(
        receiver.strip_suffix(&suffix).unwrap().to_vec(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::Two]
    );
    assert_eq!(
        receiver.strip_suffix(&PackedString::<super::Symbol, 2>::new()),
        Some(receiver.clone())
    );
    assert!(receiver.strip_suffix(&receiver).unwrap().is_empty());
    assert!(receiver.strip_suffix(&packed(&[0, 1])).is_none());
    assert!(
        receiver
            .strip_suffix(&packed(&[1, 0, 1, 0, 1, 0]))
            .is_none()
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_suffix = packed_as::<Oct, 3>(&oct_codes[20..], oct);
    let oct_stripped = oct_receiver.strip_suffix(&oct_suffix).unwrap();
    assert_eq!(
        oct_stripped.to_vec(),
        oct_codes[..20].iter().copied().map(oct).collect::<Vec<_>>()
    );
    assert_eq!(oct_stripped.bits().bit_len(), 20 * 3);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_suffix = packed_as::<WideCode, 7>(&wide_codes[8..], wide);
    let wide_stripped = wide_receiver.strip_suffix(&wide_suffix).unwrap();
    assert_eq!(
        wide_stripped.to_vec(),
        wide_codes[..8]
            .iter()
            .copied()
            .map(wide)
            .collect::<Vec<_>>()
    );
    assert_eq!(wide_stripped.bits().bit_len(), 8 * 7);
}

#[test]
fn packed_str_strip_suffix_returns_the_remaining_character_view() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let suffix = packed(&[1, 0]);
    let stripped = receiver.strip_suffix(suffix.as_packed_str()).unwrap();
    assert_eq!(
        stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::Two]
    );
    assert_eq!(stripped.char_len(), 3);
    assert_eq!(
        stripped.to_packed_string().to_vec(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::Two]
    );

    let empty_suffix_owner = packed(&[]);
    let empty_suffix = empty_suffix_owner.as_packed_str();
    assert!(receiver.strip_suffix(empty_suffix).unwrap() == receiver);
    assert!(receiver.strip_suffix(receiver).unwrap().is_empty());
    assert!(empty_suffix.strip_suffix(empty_suffix).unwrap().is_empty());

    let different_first = packed(&[0, 2]);
    let different_middle = packed(&[2, 0]);
    let different_last = packed(&[1, 1]);
    let oversized = packed(&[2, 1, 0, 1, 0, 2]);
    assert!(
        receiver
            .strip_suffix(different_first.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_suffix(different_middle.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_suffix(different_last.as_packed_str())
            .is_none()
    );
    assert!(receiver.strip_suffix(oversized.as_packed_str()).is_none());
    assert!(
        empty_suffix
            .strip_suffix(packed(&[0]).as_packed_str())
            .is_none()
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_receiver = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_suffix = packed(&[2, 1]);
    let offset_stripped = offset_receiver
        .strip_suffix(offset_suffix.as_packed_str())
        .unwrap();
    assert_eq!(
        offset_stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    assert!(
        offset_receiver
            .strip_suffix(offset_receiver.slice_until(0))
            .unwrap()
            == offset_receiver
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary
            .as_packed_str()
            .strip_suffix(binary.as_packed_str().slice_from(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::PackedSymbol::Zero]
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes
            .as_packed_str()
            .strip_suffix(bytes.as_packed_str().slice_from(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::SparseByte::Maximum]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_stripped = oct_receiver
        .strip_suffix(oct_receiver.slice_from(2))
        .unwrap();
    assert_eq!(
        oct_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![4, 5]
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_stripped = wide_receiver
        .strip_suffix(wide_receiver.slice_from(2))
        .unwrap();
    assert_eq!(
        wide_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![88, 99]
    );

    assert_eq!(offset_receiver.char_len(), 3);
    assert_eq!(offset_receiver.get(0), Some(super::Symbol::One));
    assert_eq!(offset_receiver.get(2), Some(super::Symbol::One));
}
