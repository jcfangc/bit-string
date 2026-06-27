use crate::BitString;

#[test]
fn same_source_full_views_are_equal() {
    let bits = BitString::try_from("101001").unwrap();
    assert_eq!(bits.as_bit_str(), bits.as_bit_str());
}

#[test]
fn different_sources_same_bits_are_equal() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101001").unwrap();
    assert_eq!(a.as_bit_str(), b.as_bit_str());
}

#[test]
fn different_lengths_not_equal() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101").unwrap();
    assert_ne!(a.as_bit_str(), b.as_bit_str());
}

#[test]
fn same_length_different_content_not_equal() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("111001").unwrap();
    assert_ne!(a.as_bit_str(), b.as_bit_str());
}

#[test]
fn offset_views_equal_when_content_matches() {
    let bits = BitString::try_from("110010").unwrap();
    let v1 = bits.as_bit_str().slice_from(2).slice_until(5);
    let v2 = bits.as_bit_str().slice_from(2).slice_until(5);
    assert_eq!(v1, v2);
}

#[test]
fn offset_views_not_equal_when_content_differs() {
    let bits = BitString::try_from("110010").unwrap();
    let v1 = bits.as_bit_str().slice_from(2).slice_until(5);
    let v2 = bits.as_bit_str().slice_from(1).slice_until(4);
    assert_ne!(v1, v2);
}

#[test]
fn empty_views_are_equal() {
    let a = BitString::new();
    let b = BitString::try_from("101").unwrap();
    let empty = b.as_bit_str().slice_from(0).slice_until(0);
    assert_eq!(a.as_bit_str(), empty);
    assert_eq!(empty, BitString::new().as_bit_str());
}

#[test]
fn views_across_word_boundaries() {
    let mut a = BitString::zeros(130);
    a.set(62, true);
    a.set(63, true);
    a.set(64, true);

    let mut b = BitString::zeros(130);
    b.set(62, true);
    b.set(63, true);
    b.set(64, true);

    assert_eq!(a.as_bit_str(), b.as_bit_str());

    b.set(65, true);
    assert_ne!(a.as_bit_str(), b.as_bit_str());
}
