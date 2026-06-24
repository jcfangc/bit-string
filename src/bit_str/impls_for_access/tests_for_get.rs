use int_interval::UsizeCO;

use crate::BitStr;
use crate::BitString;

// ---------------------------------------------------------------------------
// Empty views (from empty source or direct construct)
// ---------------------------------------------------------------------------

#[test]
fn empty_source_returns_none() {
    let s = BitString::new();
    let v = s.as_bit_str();
    assert!(v.is_empty());
    assert_eq!(v.get(0), None);
    assert_eq!(v.get(usize::MAX), None);
}

#[test]
fn empty_view_via_direct_construct_returns_none() {
    let s = BitString::try_from("110").unwrap();
    let v = BitStr {
        source: &s,
        start: 1,
        bit_len: 0,
    };
    assert!(v.is_empty());
    assert_eq!(v.get(0), None);
}

// ---------------------------------------------------------------------------
// In-bounds reads (full source view)
// ---------------------------------------------------------------------------

#[test]
fn full_view_reads_all_bits() {
    let s = BitString::try_from("101001").unwrap();
    let v = s.as_bit_str();

    assert_eq!(v.get(0), Some(true));
    assert_eq!(v.get(1), Some(false));
    assert_eq!(v.get(2), Some(true));
    assert_eq!(v.get(3), Some(false));
    assert_eq!(v.get(4), Some(false));
    assert_eq!(v.get(5), Some(true));
}

#[test]
fn returns_none_at_bit_len_and_beyond() {
    let s = BitString::try_from("101001").unwrap();
    let v = s.as_bit_str();

    assert_eq!(v.get(6), None); // exactly bit_len
    assert_eq!(v.get(7), None);
    assert_eq!(v.get(100), None);
    assert_eq!(v.get(usize::MAX), None);
}

#[test]
fn first_and_last_on_full_view() {
    let s = BitString::try_from("101001").unwrap();
    let v = s.as_bit_str();

    assert_eq!(v.first(), Some(true));
    assert_eq!(v.last(), Some(true));
}

// ---------------------------------------------------------------------------
// Offset views via slice (UsizeCO guarantees non-empty)
// ---------------------------------------------------------------------------

#[test]
fn offset_view_reads_correct_bits() {
    let s = BitString::try_from("00110110").unwrap(); // 0,0,1,1,0,1,1,0
    let v = s.as_bit_str().slice(UsizeCO::try_new(2, 6).unwrap());

    assert_eq!(v.bit_len(), 4);
    assert_eq!(v.start(), 2);
    assert_eq!(v.get(0), Some(true)); // src bit 2
    assert_eq!(v.get(1), Some(true)); // src bit 3
    assert_eq!(v.get(2), Some(false)); // src bit 4
    assert_eq!(v.get(3), Some(true)); // src bit 5
    assert_eq!(v.get(4), None); // beyond view
}

#[test]
fn offset_view_first_and_last() {
    let s = BitString::try_from("00011000").unwrap();
    let v = s.as_bit_str().slice(UsizeCO::try_new(3, 5).unwrap());

    assert_eq!(v.first(), Some(true));
    assert_eq!(v.last(), Some(true));
}

// ---------------------------------------------------------------------------
// Cross-word boundary views
// ---------------------------------------------------------------------------

#[test]
fn cross_word_boundary_in_offset_view() {
    let mut s = BitString::zeros(130);
    s.set(62, true);
    s.set(63, true);
    s.set(64, true);
    s.set(65, true);
    s.set(129, true);

    let v = s.as_bit_str().slice(UsizeCO::try_new(62, 130).unwrap()); // 68 bits

    assert_eq!(v.bit_len(), 68);
    assert_eq!(v.get(0), Some(true)); // src bit 62
    assert_eq!(v.get(1), Some(true)); // src bit 63
    assert_eq!(v.get(2), Some(true)); // src bit 64
    assert_eq!(v.get(3), Some(true)); // src bit 65
    assert_eq!(v.get(4), Some(false)); // src bit 66
    assert_eq!(v.get(66), Some(false)); // src bit 128
    assert_eq!(v.get(67), Some(true)); // src bit 129
    assert_eq!(v.get(68), None);
}

#[test]
fn view_starting_unaligned_and_crossing_word() {
    let mut s = BitString::zeros(130);
    s.set(60, true);
    s.set(63, true);
    s.set(64, true);
    s.set(70, true);

    let v = s.as_bit_str().slice(UsizeCO::try_new(60, 72).unwrap()); // 12 bits

    assert_eq!(v.get(0), Some(true)); // src bit 60
    assert_eq!(v.get(1), Some(false));
    assert_eq!(v.get(2), Some(false));
    assert_eq!(v.get(3), Some(true)); // src bit 63
    assert_eq!(v.get(4), Some(true)); // src bit 64
    assert_eq!(v.get(10), Some(true)); // src bit 70
    assert_eq!(v.get(11), Some(false));
    assert_eq!(v.get(12), None);
}

// ---------------------------------------------------------------------------
// Single-bit views
// ---------------------------------------------------------------------------

#[test]
fn single_bit_view() {
    let s = BitString::try_from("101").unwrap();
    let v = s.as_bit_str().slice(UsizeCO::try_new(1, 2).unwrap());

    assert_eq!(v.bit_len(), 1);
    assert_eq!(v.get(0), Some(false));
    assert_eq!(v.get(1), None);
    assert_eq!(v.first(), Some(false));
    assert_eq!(v.last(), Some(false));
}

#[test]
fn single_bit_view_at_last_position() {
    let s = BitString::try_from("101").unwrap();
    let v = s.as_bit_str().slice(UsizeCO::try_new(2, 3).unwrap());

    assert_eq!(v.bit_len(), 1);
    assert_eq!(v.get(0), Some(true));
    assert_eq!(v.get(1), None);
}
