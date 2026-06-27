use super::*;

#[test]
fn attack_display_roundtrip() {
    for s in ["", "0", "1", "01", "10", "10101", "000", "111"] {
        let bits: BitString = s.parse().unwrap();
        assert_eq!(bits.to_string(), s, "roundtrip failed for '{s}'");
    }
}

#[test]
fn attack_display_debug_consistency() {
    let bits = bs("10101");
    let debug = format!("{:?}", bits);
    assert!(debug.contains("10101"));
    assert!(debug.starts_with("BitString("));
}

#[test]
fn attack_display_empty() {
    let bits = BitString::new();
    assert_eq!(bits.to_string(), "");
    assert_eq!(format!("{:?}", bits), "BitString(\"\")");
}
