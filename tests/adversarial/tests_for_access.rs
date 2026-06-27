use super::*;

#[test]
fn attack_get_out_of_bounds() {
    let bits = bs("10101"); // len 5
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(4), Some(true));
    assert_eq!(bits.get(5), None);
    assert_eq!(bits.get(usize::MAX), None);
    assert_eq!(bits.get(usize::MAX / 2), None);
}

#[test]
fn attack_first_last_empty() {
    let bits = BitString::new();
    assert_eq!(bits.first(), None);
    assert_eq!(bits.last(), None);
}

#[test]
fn attack_get_chunk_boundaries() {
    // Empty
    let empty = BitString::new();
    assert_eq!(empty.get_chunk(0), 0);
    assert_eq!(empty.get_chunk(usize::MAX), 0);

    // Single bit
    let one = bs("1");
    assert_eq!(one.get_chunk(0), 1);
    assert_eq!(one.get_chunk(1), 0);

    // Cross-word boundary: 65 bits, start at 32
    let mut bits = BitString::zeros(65);
    bits.set_chunk(32, u64::MAX, 33);
    let chunk = bits.get_chunk(32);
    // Should have 33 valid bits
    assert_eq!(chunk & ((1u64 << 33) - 1), (1u64 << 33) - 1);

    // Start beyond len: should return 0
    let bits = bs("1111");
    assert_eq!(bits.get_chunk(4), 0);
    assert_eq!(bits.get_chunk(100), 0);
}

#[test]
fn attack_read_write_consistency() {
    // Every read should match what was written
    let mut bits = BitString::zeros(256);
    for i in 0..256 {
        let val = i % 7 == 0 || i % 3 == 0;
        bits.set(i, val);
        assert_eq!(bits.get(i), Some(val));
    }
    assert!(view_has_same_invariants(&bits));
}
