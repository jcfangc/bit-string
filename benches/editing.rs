use bit_string::BitString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

// Include operation modules at crate scope so Divan keeps the existing paths.
include!("editing/extend_bool.rs");
include!("editing/push.rs");
include!("editing/pop.rs");
include!("editing/push_bit_string.rs");
include!("editing/truncate.rs");
include!("editing/insert_middle.rs");
include!("editing/remove_middle.rs");
include!("editing/insert_remove_word_growth.rs");
include!("editing/replace_interval.rs");
include!("editing/slice.rs");

fn main() {
    divan::main();
}
