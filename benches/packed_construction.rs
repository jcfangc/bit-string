#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{LENGTHS, WIDTHS, codes, packed};

fn main() {
    divan::main();
}

#[divan::bench(name = "construct/packed", consts = WIDTHS, args = LENGTHS)]
fn construct_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| black_box(packed::<BITS>(&input)));
}

#[divan::bench(name = "construct/vec", consts = WIDTHS, args = LENGTHS)]
fn construct_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| black_box(input.iter().copied().collect::<Vec<_>>()));
}
