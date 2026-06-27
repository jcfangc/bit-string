use super::*;

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
