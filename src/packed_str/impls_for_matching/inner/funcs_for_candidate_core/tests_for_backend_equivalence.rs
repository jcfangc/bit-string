use proptest::prelude::*;

use super::{matching_lanes, scalar};

proptest! {
    #[test]
    fn selected_backend_matches_scalar(
        windows in prop::array::uniform4(any::<u64>()),
        needle_first_word in any::<u64>(),
        needle_first_word_mask in any::<u64>(),
    ) {
        let expected = scalar::matching_lanes(
            &windows,
            needle_first_word,
            needle_first_word_mask,
            4,
        );
        let actual = matching_lanes(
            &windows,
            needle_first_word,
            needle_first_word_mask,
            4,
        );
        prop_assert_eq!(actual, expected);
    }
}
