use crate::BitString;

#[test]
fn reads_aligned_chunk() {
    let bits = BitString::try_from(
        "0000000000000000000000000000000000000000000000000000000000000001\
         1000000000000000000000000000000000000000000000000000000000000000"
    ).unwrap();
    // Word 0 = 0x8000000000000000 (bit 63 set) → wait, BitString displays
    // position 0 on the left. "000...001" means bit 0 = 0, bit 63 = 1.
    // That's 0x8000000000000000 in the u64.
    // Word 1 starts with "1" = bit 64 = 1, then zeros.
    assert_eq!(bits.get_chunk(0), 0x8000_0000_0000_0000);
    // At bit 64: word 1, bit 0 = 1. That's just 1.
    assert_eq!(bits.get_chunk(64), 1);
}

#[test]
fn reads_unaligned_chunk() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);

    // At bit 62: bits 62 and 63 (from word 0) = 0b11 << 62.
    // bit 64 (from word 1) = 1.
    let chunk = bits.get_chunk(62);
    assert_eq!(chunk & 0x3, 0x3); // bits 62, 63
    assert_eq!((chunk >> 2) & 1, 1); // bit 64
}

#[test]
fn reads_past_end_returns_zeros() {
    let bits = BitString::try_from("1").unwrap();
    // bit 0 = 1, bits 1.. = 0
    assert_eq!(bits.get_chunk(0), 1);
    assert_eq!(bits.get_chunk(1), 0);
    assert_eq!(bits.get_chunk(100), 0);
}

#[test]
fn reads_empty() {
    let bits = BitString::new();
    assert_eq!(bits.get_chunk(0), 0);
}
