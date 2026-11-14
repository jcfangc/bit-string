use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[path = "packed_support/mod.rs"]
mod support;

use bit_string::{BitString, PackedString};
use divan::{Bencher, black_box};
use support::{codes, cross_word_index, cross_word_index_for_offset, packed};

include!("bit_traits/bit_fmt.rs");
include!("bit_traits/hash.rs");
include!("bit_traits/ord.rs");
include!("bit_traits/packed_cmp.rs");
include!("bit_traits/packed_hash.rs");

fn main() {
    divan::main();
}
