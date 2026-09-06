#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{LENGTHS, WIDTHS, codes, indices, packed};

fn main() {
    divan::main();
}

#[divan::bench(name = "get/sequential/packed", consts = WIDTHS, args = LENGTHS)]
fn get_sequential_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap().0;
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "get/sequential/vec", consts = WIDTHS, args = LENGTHS)]
fn get_sequential_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let input = black_box(&input);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= input[index];
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "get/random/packed", consts = WIDTHS, args = LENGTHS)]
fn get_random_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let indexes = indices(len);
    bencher.bench(|| {
        let value = black_box(&value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.get(index).unwrap().0;
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "get/random/vec", consts = WIDTHS, args = LENGTHS)]
fn get_random_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let indexes = indices(len);
    bencher.bench(|| {
        let input = black_box(&input);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= input[index];
        }
        black_box(checksum)
    });
}
