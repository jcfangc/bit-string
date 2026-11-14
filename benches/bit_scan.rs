use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

include!("bit_scan/count_ones.rs");
include!("bit_scan/any_all.rs");
include!("bit_scan/leading.rs");
include!("bit_scan/trailing.rs");

fn main() {
    divan::main();
}
