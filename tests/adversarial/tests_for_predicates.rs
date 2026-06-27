use super::*;

#[test]
fn attack_predicates_corner() {
    assert!(!BitString::new().any());
    assert!(BitString::new().all()); // vacuously true
    assert!(BitString::new().is_all_zeros());
    assert!(BitString::new().is_all_ones()); // also vacuously true

    let z = BitString::zeros(10);
    assert!(!z.any());
    assert!(!z.all());
    assert!(z.is_all_zeros());
    assert!(!z.is_all_ones());

    let o = BitString::ones(10);
    assert!(o.any());
    assert!(o.all());
    assert!(!o.is_all_zeros());
    assert!(o.is_all_ones());
}
