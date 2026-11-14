use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};

include!("bit_ops/xor.rs");
include!("bit_ops/not.rs");
include!("bit_ops/shl.rs");
include!("bit_ops/shl_assign.rs");
include!("bit_ops/shr_assign.rs");

fn main() {
    divan::main();
}
