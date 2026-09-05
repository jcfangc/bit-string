use super::*;

const VALID_LOWER: () = assert_valid_width::<1>();
const VALID_UPPER: () = assert_valid_width::<8>();
const MASK_LOWER: u8 = code_mask::<1>();
const MASK_UPPER: u8 = code_mask::<8>();

#[test]
fn accepts_the_complete_valid_width_domain() {
    let _ = VALID_LOWER;
    let _ = VALID_UPPER;

    assert_valid_width::<2>();
    assert_valid_width::<3>();
    assert_valid_width::<7>();
}

#[test]
fn returns_the_low_bits_mask_for_each_valid_width() {
    let masks = [
        code_mask::<1>(),
        code_mask::<2>(),
        code_mask::<3>(),
        code_mask::<7>(),
        code_mask::<8>(),
    ];
    assert_eq!(
        masks,
        [0b0000_0001, 0b0000_0011, 0b0000_0111, 0b0111_1111, 0xff]
    );
    assert_eq!(MASK_LOWER, masks[0]);
    assert_eq!(MASK_UPPER, masks[4]);

    for (bits, mask) in [
        (1, masks[0]),
        (2, masks[1]),
        (3, masks[2]),
        (7, masks[3]),
        (8, masks[4]),
    ] {
        assert_eq!(mask.count_ones(), bits);
        let expected = if bits == 8 {
            u8::MAX
        } else {
            ((1u16 << bits) - 1) as u8
        };
        assert_eq!(mask & !expected, 0);
    }
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn code_mask_rejects_zero_width() {
    code_mask::<0>();
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn code_mask_rejects_width_above_the_domain() {
    code_mask::<9>();
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
