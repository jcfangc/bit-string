#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{LENGTHS, WIDTHS, codes, packed};

fn main() {
    divan::main();
}

#[divan::bench(name = "iter/packed", consts = WIDTHS, args = LENGTHS)]
fn iter_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value.iter().fold(0u8, |checksum, code| checksum ^ code.0);
        black_box(checksum)
    });
}

#[divan::bench(name = "iter/vec", consts = WIDTHS, args = LENGTHS)]
fn iter_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let input = black_box(&input);
        let checksum = input.iter().fold(0u8, |checksum, &code| checksum ^ code);
        black_box(checksum)
    });
}
