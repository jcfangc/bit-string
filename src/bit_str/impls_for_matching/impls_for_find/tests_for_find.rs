use crate::{AnchorHint, BitString, ContainsOptions, FindOptions, LocationHint};

// ---------------------------------------------------------------------------
// contains
// ---------------------------------------------------------------------------

#[test]
fn contains_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("01").unwrap();
    assert!(v.contains_str_with(bs.as_bit_str(), crate::ContainsOptions::default()));
    let bs = BitString::try_from("111").unwrap();
    assert!(!v.contains_str_with(bs.as_bit_str(), crate::ContainsOptions::default()));
}

#[test]
fn contains_empty_needle() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let empty = BitString::new();
    assert!(v.contains_str_with(empty.as_bit_str(), crate::ContainsOptions::default()));
}

#[test]
fn contains_longer_than_view_returns_false() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("101").unwrap();
    assert!(!v.contains_str_with(bs.as_bit_str(), crate::ContainsOptions::default()));
}

#[test]
fn contains_on_offset_view() {
    let bits = BitString::try_from("110010").unwrap();
    let v = bits.as_bit_str().slice_from(1).slice_until(5);
    let bs = BitString::try_from("001").unwrap();
    assert!(v.contains_str_with(bs.as_bit_str(), crate::ContainsOptions::default()));
    let bs = BitString::try_from("11").unwrap();
    assert!(!v.contains_str_with(bs.as_bit_str(), crate::ContainsOptions::default()));
}

// ---------------------------------------------------------------------------
// find
// ---------------------------------------------------------------------------

#[test]
fn find_first_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(0)
    );
    let bs = BitString::try_from("01").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(1)
    );
    let bs = BitString::try_from("111").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        None
    );
}

#[test]
fn find_empty_needle_returns_zero() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let empty = BitString::new();
    assert_eq!(
        v.find_str_with(empty.as_bit_str(), crate::FindOptions::default()),
        Some(0)
    );
}

#[test]
fn find_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        None
    );
}

#[test]
fn find_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    let v = bits.as_bit_str().slice_from(2).slice_until(7);
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(2)
    );
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(0)
    );
}

#[test]
fn find_at_end_of_view() {
    let bits = BitString::try_from("11100").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(3)
    );
}

// ---------------------------------------------------------------------------
// rfind
// ---------------------------------------------------------------------------

#[test]
fn rfind_last_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(
        v.rfind_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(6)
    );
}

#[test]
fn rfind_empty_needle_returns_bit_len() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let empty = BitString::new();
    assert_eq!(
        v.rfind_str_with(empty.as_bit_str(), crate::FindOptions::default()),
        Some(5)
    );
}

#[test]
fn rfind_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(
        v.rfind_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        None
    );
}

#[test]
fn rfind_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    let v = bits.as_bit_str().slice_from(2).slice_until(7);
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(
        v.rfind_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(3)
    );
}

#[test]
fn find_and_rfind_needle_appears_once() {
    let bits = BitString::try_from("11010").unwrap();
    let v = bits.as_bit_str();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(
        v.find_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(1)
    );
    assert_eq!(
        v.rfind_str_with(bs.as_bit_str(), crate::FindOptions::default()),
        Some(1)
    );
}

#[test]
fn with_entrypoints_preserve_bit_search_semantics() {
    let source = BitString::try_from("11101100101101001").unwrap();
    let bits = source.as_bit_str().slice_from(2).slice_until(15);
    let needle_owner = BitString::try_from("1011").unwrap();
    let needle_view_owner = BitString::try_from("00110").unwrap();
    let needle_view = needle_view_owner.as_bit_str().slice_from(2).slice_until(5);
    let anchors = [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix];
    let locations = [
        LocationHint::Unknown,
        LocationHint::NearStart,
        LocationHint::NearEnd,
    ];

    // The bit-view contents are 1011001011010 and the needle is 110.
    // These constants are the independent expected first/last positions.
    let expected_find = Some(2);
    let expected_rfind = Some(8);

    for anchor in anchors {
        let options = FindOptions::default().with_anchor(anchor);
        assert_eq!(bits.find_str_with(needle_view, options), expected_find);
        assert_eq!(bits.rfind_str_with(needle_view, options), expected_rfind);
        assert_eq!(bits.find_string_with(&needle_owner, options), Some(0));
        assert_eq!(bits.rfind_string_with(&needle_owner, options), Some(6));

        for location in locations {
            let options = ContainsOptions::default()
                .with_anchor(anchor)
                .with_location(location);
            assert!(bits.contains_str_with(needle_view, options));
            assert!(bits.contains_string_with(&needle_owner, options));
        }
    }
}
