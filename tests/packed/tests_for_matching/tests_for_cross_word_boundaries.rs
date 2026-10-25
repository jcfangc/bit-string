use super::{Oct, WideCode, assert_matching, oct, wide};
use bit_string::traits::PackedChar;
use proptest::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TestCode<const BITS: u8>(u8);

impl<const BITS: u8> PackedChar<BITS> for TestCode<BITS> {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        ((1..=8).contains(&BITS) && u16::from(code) < (1_u16 << BITS)).then_some(Self(code))
    }
}

fn test_code<const BITS: u8>(code: u8) -> TestCode<BITS> {
    assert!(u16::from(code) < (1_u16 << BITS));
    TestCode(code)
}

proptest! {
#[test]
    fn three_and_seven_bit_matching_crosses_word_boundaries(
        haystack3 in prop::collection::vec(0u8..=7, 22..=128),
        needle3 in prop::collection::vec(0u8..=7, 0..=32),
        haystack7 in prop::collection::vec(0u8..=127, 10..=128),
        needle7 in prop::collection::vec(0u8..=127, 0..=16),
    ) {
        assert_matching::<Oct, 3>(&haystack3, &needle3, oct);
        assert_matching::<WideCode, 7>(&haystack7, &needle7, wide);
    }

    #[test]
    fn four_five_and_six_bit_search_crosses_word_boundaries(
        haystack4 in prop::collection::vec(0u8..=15, 16..=128),
        needle4 in prop::collection::vec(0u8..=15, 0..=32),
        haystack5 in prop::collection::vec(0u8..=31, 13..=128),
        needle5 in prop::collection::vec(0u8..=31, 0..=32),
        haystack6 in prop::collection::vec(0u8..=63, 11..=128),
        needle6 in prop::collection::vec(0u8..=63, 0..=32),
    ) {
        assert_matching::<TestCode<4>, 4>(&haystack4, &needle4, test_code::<4>);
        assert_matching::<TestCode<5>, 5>(&haystack5, &needle5, test_code::<5>);
        assert_matching::<TestCode<6>, 6>(&haystack6, &needle6, test_code::<6>);
    }
}
