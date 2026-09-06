use super::*;

#[test]
fn packed_slice_returns_a_clamped_character_aligned_owner() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let string = PackedString::from_chars(original.clone());

    let sliced = string.slice(UsizeCO::checked_from_start_len(9, 5).unwrap());
    assert_eq!(sliced.to_vec(), original[9..14].to_vec());
    assert_eq!(sliced.char_len(), 5);
    assert_eq!(sliced.bits().bit_len(), 5 * 3);
    assert_eq!(string.to_vec(), original);

    let tail = string.slice(UsizeCO::checked_from_start_len(20, 9).unwrap());
    assert_eq!(tail.to_vec(), original[20..].to_vec());
    assert_eq!(tail.bits().bit_len(), 2 * 3);

    let empty = string.slice(UsizeCO::checked_from_start_len(99, 4).unwrap());
    assert!(empty.is_empty());
    assert!(empty.bits().words().is_empty());

    let wide_values: Vec<_> = (0..16)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let wide = PackedString::from_chars(wide_values.clone());
    let wide_sliced = wide.slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    assert_eq!(wide_sliced.to_vec(), wide_values[8..13].to_vec());
    assert_eq!(wide_sliced.bits().bit_len(), 5 * 7);
}

#[test]
fn packed_slice_from_returns_the_clamped_suffix() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let string = PackedString::from_chars(original.clone());

    let suffix = string.slice_from(9);
    assert_eq!(suffix.to_vec(), original[9..].to_vec());
    assert_eq!(suffix.bits().bit_len(), (original.len() - 9) * 3);
    assert_eq!(string.to_vec(), original);

    assert_eq!(string.slice_from(0).to_vec(), original);
    let empty = string.slice_from(usize::MAX);
    assert!(empty.is_empty());
    assert!(empty.bits().words().is_empty());

    let wide_values: Vec<_> = (0..16)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let wide = PackedString::from_chars(wide_values.clone());
    let wide_suffix = wide.slice_from(8);
    assert_eq!(wide_suffix.to_vec(), wide_values[8..].to_vec());
    assert_eq!(wide_suffix.bits().bit_len(), 8 * 7);
}

#[test]
fn packed_slice_until_returns_the_clamped_prefix() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let string = PackedString::from_chars(original.clone());

    let prefix = string.slice_until(9);
    assert_eq!(prefix.to_vec(), original[..9].to_vec());
    assert_eq!(prefix.bits().bit_len(), 9 * 3);
    assert_eq!(string.to_vec(), original);

    assert!(string.slice_until(0).is_empty());
    assert_eq!(string.slice_until(usize::MAX).to_vec(), original);

    let wide_values: Vec<_> = (0..16)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let wide = PackedString::from_chars(wide_values.clone());
    let wide_prefix = wide.slice_until(8);
    assert_eq!(wide_prefix.to_vec(), wide_values[..8].to_vec());
    assert_eq!(wide_prefix.bits().bit_len(), 8 * 7);
}

#[test]
fn packed_split_off_returns_a_character_aligned_suffix() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut string = PackedString::from_chars(original.clone());
    let suffix = string.split_off(21);
    assert_eq!(string.to_vec(), original[..21].to_vec());
    assert_eq!(suffix.to_vec(), original[21..].to_vec());
    assert_eq!(string.bits().bit_len(), 21 * 3);
    assert_eq!(suffix.bits().bit_len(), 3);

    let mut at_end = PackedString::from_chars(original.clone());
    let empty = at_end.split_off(usize::MAX);
    assert_eq!(at_end.to_vec(), original);
    assert!(empty.is_empty());
    assert!(empty.bits().words().is_empty());

    let mut at_start = PackedString::from_chars(original.clone());
    let all = at_start.split_off(0);
    assert!(at_start.is_empty());
    assert_eq!(all.to_vec(), original);
    assert_eq!(all.bits().bit_len(), 22 * 3);

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let mut wide = PackedString::from_chars(wide_values.clone());
    let wide_suffix = wide.split_off(8);
    assert_eq!(wide.to_vec(), wide_values[..8].to_vec());
    assert_eq!(wide_suffix.to_vec(), wide_values[8..].to_vec());
    assert_eq!(wide.bits().bit_len(), 8 * 7);
    assert_eq!(wide_suffix.bits().bit_len(), 2 * 7);
}
