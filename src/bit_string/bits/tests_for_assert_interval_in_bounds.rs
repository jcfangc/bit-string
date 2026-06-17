use int_interval::UsizeCO;

use super::*;

#[test]
fn accepts_interval_strictly_inside_len() {
    let interval = UsizeCO::try_new(2, 5).unwrap();

    assert_interval_in_bounds(interval, 8);
}

#[test]
fn accepts_interval_ending_at_len() {
    let interval = UsizeCO::try_new(2, 8).unwrap();

    assert_interval_in_bounds(interval, 8);
}

#[test]
fn accepts_full_interval() {
    let interval = UsizeCO::try_new(0, 8).unwrap();

    assert_interval_in_bounds(interval, 8);
}

#[test]
#[should_panic(expected = "bit string interval out of bounds")]
fn panics_when_end_excl_exceeds_len() {
    let interval = UsizeCO::try_new(2, 9).unwrap();

    assert_interval_in_bounds(interval, 8);
}

#[test]
#[should_panic(expected = "bit string interval out of bounds")]
fn panics_when_non_empty_interval_is_used_against_empty_len() {
    let interval = UsizeCO::try_new(0, 1).unwrap();

    assert_interval_in_bounds(interval, 0);
}
