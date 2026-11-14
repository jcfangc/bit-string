#[path = "packed_support/mod.rs"]
mod support;

use bit_string::PackedStr;
use bit_string::PackedString;
use divan::{Bencher, black_box};
use support::Code;
use support::{codes, mix64, packed, unaligned_index};

include!("packed_search_candidates.rs");
include!("packed_native.rs");

fn main() {
    divan::main();
}
