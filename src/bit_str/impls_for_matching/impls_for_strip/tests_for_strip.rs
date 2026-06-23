use crate::BitString;

// ---------------------------------------------------------------------------
// strip_prefix
// ---------------------------------------------------------------------------

#[test]
fn strip_prefix_removes_matching_prefix() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();

    let rest = v
        .strip_prefix(&BitString::try_from("101").unwrap())
        .unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true)); // original bit 3
    assert_eq!(rest.get(1), Some(false)); // original bit 4
    assert_eq!(rest.get(2), Some(false)); // original bit 5
}

#[test]
fn strip_prefix_non_matching_returns_none() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();

    assert!(
        v.strip_prefix(&BitString::try_from("11").unwrap())
            .is_none()
    );
}

#[test]
fn strip_prefix_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();

    let rest = v.strip_prefix(&BitString::new()).unwrap();
    assert_eq!(rest.bit_len(), v.bit_len());
}

#[test]
fn strip_prefix_entire_view() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bitstr();

    let rest = v
        .strip_prefix(&BitString::try_from("101").unwrap())
        .unwrap();
    assert_eq!(rest.bit_len(), 0);
}

#[test]
fn strip_prefix_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // view bits 1..6 → 1 0 1 0 1
    let v = bits.as_bitstr().slice_from(1).slice_until(5);

    let rest = v.strip_prefix(&BitString::try_from("10").unwrap()).unwrap();
    // remaining: "101" (3 bits)
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(true));
}

// ---------------------------------------------------------------------------
// strip_suffix
// ---------------------------------------------------------------------------

#[test]
fn strip_suffix_removes_matching_suffix() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();

    let rest = v
        .strip_suffix(&BitString::try_from("100").unwrap())
        .unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true)); // bit 0
    assert_eq!(rest.get(1), Some(false)); // bit 1
    assert_eq!(rest.get(2), Some(true)); // bit 2
}

#[test]
fn strip_suffix_non_matching_returns_none() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();

    assert!(
        v.strip_suffix(&BitString::try_from("10").unwrap())
            .is_none()
    );
}

#[test]
fn strip_suffix_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();

    let rest = v.strip_suffix(&BitString::new()).unwrap();
    assert_eq!(rest.bit_len(), v.bit_len());
}

#[test]
fn strip_suffix_entire_view() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bitstr();

    let rest = v
        .strip_suffix(&BitString::try_from("101").unwrap())
        .unwrap();
    assert_eq!(rest.bit_len(), 0);
}

#[test]
fn strip_suffix_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // view bits 1..6 → 1 0 1 0 1
    let v = bits.as_bitstr().slice_from(1).slice_until(5);

    let rest = v.strip_suffix(&BitString::try_from("01").unwrap()).unwrap();
    // remaining: "101" (3 bits)
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(true));
}
