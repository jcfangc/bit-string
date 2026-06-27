use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_iter_empty() {
    let bits = BitString::new();
    let iter = bits.iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert!(bits.to_bool_vec().is_empty());
}

#[test]
fn attack_iter_double_ended() {
    let bits = bs("10101");
    let mut iter = bits.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(true));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn attack_iter_exact_size() {
    for len in [0, 1, 64, 65, 128] {
        let bits = BitString::ones(len);
        let iter = bits.iter();
        assert_eq!(iter.len(), len);
        assert_eq!(iter.count(), len);
    }
}

#[test]
fn attack_iter_consistency_with_get() {
    let bits = bs("1100101");
    for (i, bit) in bits.iter().enumerate() {
        assert_eq!(Some(bit), bits.get(i));
    }
}

// ===========================================================================
// J. BitStr::iter on unaligned views
// ===========================================================================

#[test]
fn attack_bitstr_iter_unaligned() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "10101",
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(11, 5).unwrap());
    let collected: Vec<bool> = view.iter().collect();
    assert_eq!(collected, vec![false, true, false, true, false]);
}

#[test]
fn attack_bitstr_iter_double_ended_unaligned() {
    let a = bs(&cat(&[
        "1".repeat(5).as_str(),
        "00110011",
        "1".repeat(5).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(6, 6).unwrap());
    let mut iter = view.iter();
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next_back(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), None);
}
