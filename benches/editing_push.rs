use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench_group]
mod push {
    use super::*;

    #[divan::bench_group]
    mod len_65 {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65);
        }
    }

    #[divan::bench_group]
    mod len_65536 {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65_536);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65_536);
        }
    }
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.push(true);
        black_box(bits)
    });
}

fn bench_string(bencher: Bencher, len: usize) {
    let input = make_string(len);

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        string.push('1');
        black_box(string)
    });
}

#[inline]
fn make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| bit_at(index)).collect()
}

fn make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);

    for index in 0..len {
        out.push(bit_char(bit_at(index)));
    }

    out
}

#[inline]
fn bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn bit_at(index: usize) -> bool {
    mix64(index as u64) & 1 != 0
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
