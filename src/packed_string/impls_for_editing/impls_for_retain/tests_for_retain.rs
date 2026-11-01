use crate::traits::PackedChar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Code<const BITS: u8>(u8);

impl<const BITS: u8> PackedChar<BITS> for Code<BITS> {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code <= crate::code_mask::<BITS>()).then_some(Self(code))
    }
}

fn assert_retain_preserves_order<const BITS: u8>() {
    let codes = (0usize..193)
        .map(|index| (index.wrapping_mul(37).wrapping_add(11) as u8) & crate::code_mask::<BITS>())
        .collect::<alloc::vec::Vec<_>>();
    let mut value =
        crate::PackedString::<Code<BITS>, BITS>::from_chars(codes.iter().copied().map(Code));
    let expected = codes
        .iter()
        .copied()
        .filter(|code| code.count_ones() % 2 == 0)
        .collect::<alloc::vec::Vec<_>>();

    value.retain(|character| character.code().count_ones() % 2 == 0);

    assert_eq!(
        value
            .iter()
            .map(|character| character.code())
            .collect::<alloc::vec::Vec<_>>(),
        expected
    );
}

fn assert_retain_all_and_none<const BITS: u8>() {
    let codes = (0usize..193)
        .map(|index| (index.wrapping_mul(37).wrapping_add(11) as u8) & crate::code_mask::<BITS>())
        .collect::<alloc::vec::Vec<_>>();
    let input =
        crate::PackedString::<Code<BITS>, BITS>::from_chars(codes.iter().copied().map(Code));

    let mut all = input.clone();
    all.retain(|_| true);
    assert_eq!(
        all.iter()
            .map(|character| character.code())
            .collect::<alloc::vec::Vec<_>>(),
        codes
    );

    let mut none = input;
    none.retain(|_| false);
    assert!(none.is_empty());
}

#[test]
fn retains_in_order_for_all_widths() {
    assert_retain_preserves_order::<1>();
    assert_retain_preserves_order::<2>();
    assert_retain_preserves_order::<3>();
    assert_retain_preserves_order::<4>();
    assert_retain_preserves_order::<5>();
    assert_retain_preserves_order::<6>();
    assert_retain_preserves_order::<7>();
    assert_retain_preserves_order::<8>();
}

#[test]
fn retain_all_and_none_work_for_all_widths() {
    assert_retain_all_and_none::<1>();
    assert_retain_all_and_none::<2>();
    assert_retain_all_and_none::<3>();
    assert_retain_all_and_none::<4>();
    assert_retain_all_and_none::<5>();
    assert_retain_all_and_none::<6>();
    assert_retain_all_and_none::<7>();
    assert_retain_all_and_none::<8>();
}

#[test]
fn predicate_panic_leaves_a_valid_editable_packed_string() {
    extern crate std;

    let original = (0u8..140)
        .map(|code| code & 0b11)
        .collect::<alloc::vec::Vec<_>>();
    let mut value =
        crate::PackedString::<Code<2>, 2>::from_chars(original.iter().copied().map(Code));
    let expected_partial = original
        .iter()
        .take(128)
        .copied()
        .filter(|code| code & 1 == 0)
        .chain(original.iter().skip(64).copied())
        .collect::<alloc::vec::Vec<_>>();
    let mut calls = 0;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        value.retain(|character| {
            if calls == 133 {
                panic!("predicate panic");
            }
            calls += 1;
            character.code() & 1 == 0
        });
    }));

    assert!(result.is_err());
    assert_eq!(calls, 133);
    assert_eq!(value.char_len(), original.len());
    assert_eq!(
        value
            .iter()
            .map(|character| character.code())
            .collect::<alloc::vec::Vec<_>>(),
        expected_partial
    );

    value.push(Code(3));
    assert_eq!(value.get(original.len()), Some(Code(3)));
}
