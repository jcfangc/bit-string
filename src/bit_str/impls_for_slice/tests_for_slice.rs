use int_interval::UsizeCO;

use crate::BitString;

// ---------------------------------------------------------------------------
// slice (UsizeCO)
// ---------------------------------------------------------------------------

#[test]
fn slice_full_range_returns_identical_view() {
    let bits = BitString::try_from("1010011100").unwrap();
    let v = bits.as_bit_str();
    let s = v.slice(UsizeCO::try_new(0, 10).unwrap());

    assert_eq!(s.bit_len(), v.bit_len());
    assert_eq!(s.start(), v.start());
    // Bits are identical.
    for i in 0..v.bit_len() {
        assert_eq!(s.get(i), v.get(i), "bit {i}");
    }
}

#[test]
fn slice_subrange_within_bounds() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    // Slice bits 2..6 → "1100"
    let s = v.slice(UsizeCO::try_new(2, 6).unwrap());
    assert_eq!(s.bit_len(), 4);
    assert_eq!(s.get(0), Some(true));
    assert_eq!(s.get(1), Some(true));
    assert_eq!(s.get(2), Some(false));
    assert_eq!(s.get(3), Some(false));
}

#[test]
fn slice_end_beyond_view_is_clamped() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    // [2, 100) → clamped to [2, 5)
    let s = v.slice(UsizeCO::try_new(2, 100).unwrap());
    assert_eq!(s.bit_len(), 3);
    assert_eq!(s.get(0), v.get(2));
    assert_eq!(s.get(1), v.get(3));
    assert_eq!(s.get(2), v.get(4));
}

#[test]
fn slice_start_beyond_view_is_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    // [10, 20) → start clamped to 5, end clamped to 5 → empty
    let s = v.slice(UsizeCO::try_new(10, 20).unwrap());
    assert_eq!(s.bit_len(), 0);
}

#[test]
fn slice_single_bit() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice(UsizeCO::try_new(0, 1).unwrap());
    assert_eq!(s.bit_len(), 1);
    assert_eq!(s.get(0), Some(true));
}

#[test]
fn slice_last_bit() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice(UsizeCO::try_new(4, 5).unwrap());
    assert_eq!(s.bit_len(), 1);
    assert_eq!(s.get(0), Some(false)); // last bit is 0
}

#[test]
fn slice_chaining() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    // First slice bits 2..7 → "11000"
    let s1 = v.slice(UsizeCO::try_new(2, 7).unwrap());
    assert_eq!(s1.bit_len(), 5);

    // Then slice bits 1..4 of s1 → bits (2+1)..(2+4) of original → "100"
    let s2 = s1.slice(UsizeCO::try_new(1, 4).unwrap());
    assert_eq!(s2.bit_len(), 3);
    assert_eq!(s2.get(0), v.get(3));
    assert_eq!(s2.get(1), v.get(4));
    assert_eq!(s2.get(2), v.get(5));
}

// ---------------------------------------------------------------------------
// slice_from
// ---------------------------------------------------------------------------

#[test]
fn slice_from_zero_returns_identical_view() {
    let bits = BitString::try_from("1010011100").unwrap();
    let v = bits.as_bit_str();
    let s = v.slice_from(0);

    assert_eq!(s.bit_len(), v.bit_len());
    assert_eq!(s.start(), v.start());
}

#[test]
fn slice_from_mid() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_from(4);
    assert_eq!(s.bit_len(), 4);
    // Original bits 4..8 → "0000"
    for i in 0..4 {
        assert_eq!(s.get(i), Some(false));
    }
}

#[test]
fn slice_from_returns_empty_at_boundary() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_from(5);
    assert_eq!(s.bit_len(), 0);
}

#[test]
fn slice_from_beyond_boundary_returns_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_from(100);
    assert_eq!(s.bit_len(), 0);
}

#[test]
fn slice_from_chaining() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    // slice_from(2) → bits 2..8 → "110000"
    let s1 = v.slice_from(2);
    assert_eq!(s1.bit_len(), 6);

    // slice_from(3) of s1 → bits 5..8 → "000"
    let s2 = s1.slice_from(3);
    assert_eq!(s2.bit_len(), 3);
    assert_eq!(s2.get(0), v.get(5));
    assert_eq!(s2.get(1), v.get(6));
    assert_eq!(s2.get(2), v.get(7));
}

// ---------------------------------------------------------------------------
// slice_until
// ---------------------------------------------------------------------------

#[test]
fn slice_until_full_returns_identical_view() {
    let bits = BitString::try_from("1010011100").unwrap();
    let v = bits.as_bit_str();
    let s = v.slice_until(10);

    assert_eq!(s.bit_len(), v.bit_len());
    assert_eq!(s.start(), v.start());
}

#[test]
fn slice_until_mid() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_until(4);
    assert_eq!(s.bit_len(), 4);
    assert_eq!(s.get(0), Some(true));
    assert_eq!(s.get(1), Some(true));
    assert_eq!(s.get(2), Some(true));
    assert_eq!(s.get(3), Some(true));
}

#[test]
fn slice_until_zero_returns_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_until(0);
    assert_eq!(s.bit_len(), 0);
}

#[test]
fn slice_until_beyond_boundary_is_clamped() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();

    let s = v.slice_until(100);
    assert_eq!(s.bit_len(), 5);
}

#[test]
fn slice_until_chaining() {
    let bits = BitString::try_from("11110000").unwrap();
    let v = bits.as_bit_str();

    // slice_until(6) → bits 0..6 → "111100"
    let s1 = v.slice_until(6);
    assert_eq!(s1.bit_len(), 6);

    // slice_until(3) of s1 → bits 0..3 → "111"
    let s2 = s1.slice_until(3);
    assert_eq!(s2.bit_len(), 3);
    assert_eq!(s2.get(0), Some(true));
    assert_eq!(s2.get(1), Some(true));
    assert_eq!(s2.get(2), Some(true));
}

// ---------------------------------------------------------------------------
// Mixed slice / slice_from / slice_until
// ---------------------------------------------------------------------------

#[test]
fn mixed_slice_and_slice_from() {
    let bits = BitString::try_from("1011010011").unwrap();
    let v = bits.as_bit_str();

    // slice(2..8) → bits 2..8 → "110100"
    let s1 = v.slice(UsizeCO::try_new(2, 8).unwrap());
    assert_eq!(s1.bit_len(), 6);

    // slice_from(2) of s1 → bits 4..8 → "0100"
    let s2 = s1.slice_from(2);
    assert_eq!(s2.bit_len(), 4);
}

#[test]
fn mixed_slice_and_slice_until() {
    let bits = BitString::try_from("1011010011").unwrap();
    let v = bits.as_bit_str();

    // slice(2..8) → bits 2..8 → "110100"
    let s1 = v.slice(UsizeCO::try_new(2, 8).unwrap());

    // slice_until(4) of s1 → bits 2..6 → "1101"
    let s2 = s1.slice_until(4);
    assert_eq!(s2.bit_len(), 4);
}

// ---------------------------------------------------------------------------
// Invariants on offset views
// ---------------------------------------------------------------------------

#[test]
fn slice_on_already_offset_view() {
    let bits = BitString::try_from("111100001010").unwrap();
    let v = bits.as_bit_str();

    // Offset view from bit 3
    let off = v.slice(UsizeCO::try_new(3, 12).unwrap());
    assert_eq!(off.bit_len(), 9);
    assert_eq!(off.get(0), v.get(3));
    assert_eq!(off.get(8), v.get(11));

    // Slice again within the offset view
    let s = off.slice(UsizeCO::try_new(2, 7).unwrap());
    assert_eq!(s.bit_len(), 5);
    // Should map back to original bits 5..10
    for i in 0..5 {
        assert_eq!(s.get(i), v.get(5 + i), "bit {i}");
    }
}

#[test]
fn slice_from_on_already_offset_view() {
    let bits = BitString::try_from("111100001010").unwrap();
    let v = bits.as_bit_str();

    let off = v.slice(UsizeCO::try_new(3, 12).unwrap());
    let s = off.slice_from(4);
    assert_eq!(s.get(0), v.get(7));
}

#[test]
fn slice_until_on_already_offset_view() {
    let bits = BitString::try_from("111100001010").unwrap();
    let v = bits.as_bit_str();

    let off = v.slice(UsizeCO::try_new(3, 12).unwrap());
    let s = off.slice_until(4);
    assert_eq!(s.get(3), v.get(6));
}
