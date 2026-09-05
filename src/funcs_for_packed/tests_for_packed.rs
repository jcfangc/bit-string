use super::*;

const VALID_LOWER: () = assert_valid_width::<1>();
const VALID_UPPER: () = assert_valid_width::<8>();

#[test]
fn accepts_the_complete_valid_width_domain() {
    let _ = VALID_LOWER;
    let _ = VALID_UPPER;

    assert_valid_width::<2>();
    assert_valid_width::<3>();
    assert_valid_width::<7>();
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn rejects_zero_width() {
    assert_valid_width::<0>();
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn rejects_first_width_above_the_domain() {
    assert_valid_width::<9>();
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn rejects_maximum_u8_width() {
    assert_valid_width::<{ u8::MAX }>();
}
