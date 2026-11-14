use bit_string::BitString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

include!("matching/find.rs");
include!("matching/rfind.rs");
include!("matching/matches_at.rs");
include!("matching/starts_with.rs");
include!("matching/ends_with.rs");
include!("matching/strip_prefix.rs");

fn main() {
    divan::main();
}
