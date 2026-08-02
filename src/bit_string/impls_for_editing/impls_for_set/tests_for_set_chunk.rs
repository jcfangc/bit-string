use crate::BitString;

#[test]
fn sets_aligned_chunk() {
    let mut bits = BitString::zeros(128);
    bits.set_chunk(0, 0xDEAD_BEEF, 32);
    assert_eq!(bits.get_chunk(0) & 0xFFFF_FFFF, 0xDEAD_BEEF);
}

#[test]
fn sets_unaligned_chunk() {
    let mut bits = BitString::zeros(130);
    bits.set_chunk(62, 0b111, 3);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(61), Some(false));
    assert_eq!(bits.get(65), Some(false));
}

#[test]
fn set_chunk_overwrites_existing_bits() {
    let mut bits = BitString::ones(4);
    bits.set_chunk(1, 0b01, 2);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(1), Some(true));
    assert_eq!(bits.get(2), Some(false));
    assert_eq!(bits.get(3), Some(true));
}

#[test]
fn overwrite_crosses_a_word_boundary() {
    let mut bits = BitString::ones(130);
    bits.set_chunk(62, 0b001, 3);

    assert_eq!(bits.get(61), Some(true));
    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
}

#[test]
fn high_bits_of_value_are_masked() {
    let mut bits = BitString::zeros(8);
    bits.set_chunk(0, 0xFFFF_FFFF_FFFF_FFFF, 2);
    assert_eq!(bits.get_chunk(0) & 0xFC, 0);
    assert_eq!(bits.get_chunk(0) & 0x3, 0x3);
}

#[test]
fn writing_past_len_is_noop() {
    let mut bits = BitString::zeros(1);
    bits.set_chunk(100, 0xFF, 8);
    assert_eq!(bits.get(0), Some(false));
}
