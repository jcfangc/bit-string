use int_interval::UsizeCO;

use crate::BitStr;
use crate::BitString;
use crate::low_mask;

// ---------------------------------------------------------------------------
// Full source view
// ---------------------------------------------------------------------------

#[test]
fn aligned_chunk_from_start() {
    let s = BitString::try_from("1").unwrap();
    let v = s.as_bitstr();
    assert_eq!(v.get_chunk(0), 1);
}

#[test]
fn aligned_chunk_spanning_two_words() {
    let mut s = BitString::zeros(130);
    s.set(63, true); // last bit of word 0
    s.set(64, true); // first bit of word 1

    let v = s.as_bitstr();

    let chunk = v.get_chunk(63);
    assert_eq!(chunk & 1, 1); // bit 63 → LSB of chunk
    assert_eq!((chunk >> 1) & 1, 1); // bit 64
    assert_eq!(chunk >> 2, 0); // nothing beyond valid bits
}

// ---------------------------------------------------------------------------
// Unaligned offset views
// ---------------------------------------------------------------------------

#[test]
fn unaligned_chunk_in_offset_view() {
    let mut s = BitString::zeros(130);
    s.set(61, true);
    s.set(62, true);
    s.set(63, true);
    s.set(64, true); // crosses into word 1

    let v = s.as_bitstr().slice(UsizeCO::try_new(61, 66).unwrap()); // 5 bits

    // Bits 61..65: 1,1,1,1,0 → 0b01111 = 15
    assert_eq!(v.get_chunk(0), 0b01111);
    // Remaining 3 bits: 1,1,0 → 0b011 = 3
    assert_eq!(v.get_chunk(2), 0b011);
}

// ---------------------------------------------------------------------------
// Edge: reading at or beyond view boundary
// ---------------------------------------------------------------------------

#[test]
fn chunk_at_bit_len_returns_zero() {
    let s = BitString::try_from("11111111").unwrap();
    let v = s.as_bitstr().slice(UsizeCO::try_new(2, 5).unwrap()); // 3 bits

    assert_eq!(v.get_chunk(3), 0); // exactly bit_len
    assert_eq!(v.get_chunk(4), 0); // beyond
    assert_eq!(v.get_chunk(100), 0);
    assert_eq!(v.get_chunk(usize::MAX), 0);
}

#[test]
fn chunk_does_not_leak_bits_beyond_view() {
    let s = BitString::ones(130);
    let v = s.as_bitstr().slice(UsizeCO::try_new(60, 70).unwrap()); // 10 bits

    let chunk = v.get_chunk(5); // 5 remaining valid bits
    assert_eq!(chunk, low_mask(5)); // exactly 5 ones, nothing leaked
    assert_eq!(chunk >> 5, 0);
}

#[test]
fn chunk_at_last_bit() {
    let mut s = BitString::zeros(130);
    s.set(129, true);
    let v = s.as_bitstr().slice(UsizeCO::try_new(128, 130).unwrap()); // 2 bits

    assert_eq!(v.get_chunk(1), 1); // just bit 129
    assert_eq!(v.get_chunk(2), 0); // beyond view
}

// ---------------------------------------------------------------------------
// Empty view
// ---------------------------------------------------------------------------

#[test]
fn empty_view_chunk_always_zero() {
    let s = BitString::ones(64);
    let v = BitStr {
        source: &s,
        start: 10,
        bit_len: 0,
    };

    assert_eq!(v.get_chunk(0), 0);
    assert_eq!(v.get_chunk(1), 0);
}
