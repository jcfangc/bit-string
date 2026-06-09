use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

#[divan::bench_group]
mod replace_interval {
    use super::*;

    #[divan::bench_group]
    mod len_65_same_len {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65, ReplaceShape::SameLen);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65, ReplaceShape::SameLen);
        }
    }

    #[divan::bench_group]
    mod len_65_shorter {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65, ReplaceShape::Shorter);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65, ReplaceShape::Shorter);
        }
    }

    #[divan::bench_group]
    mod len_65_longer {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65, ReplaceShape::Longer);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65, ReplaceShape::Longer);
        }
    }

    #[divan::bench_group]
    mod len_65536_same_len {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65_536, ReplaceShape::SameLen);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65_536, ReplaceShape::SameLen);
        }
    }

    #[divan::bench_group]
    mod len_65536_shorter {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65_536, ReplaceShape::Shorter);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65_536, ReplaceShape::Shorter);
        }
    }

    #[divan::bench_group]
    mod len_65536_longer {
        use super::*;

        #[divan::bench]
        fn bit_string(bencher: Bencher) {
            bench_bit_string(bencher, 65_536, ReplaceShape::Longer);
        }

        #[divan::bench]
        fn string(bencher: Bencher) {
            bench_string(bencher, 65_536, ReplaceShape::Longer);
        }
    }
}

#[derive(Clone, Copy)]
enum ReplaceShape {
    SameLen,
    Shorter,
    Longer,
}

fn bench_bit_string(bencher: Bencher, len: usize, shape: ReplaceShape) {
    let input = make_bit_string(len);
    let start = len / 3;
    let old_len = chunk_len(len);
    let replacement = make_bit_string(replacement_len(old_len, shape));
    let interval = iv(start, old_len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.replace_interval(black_box(interval), black_box(&replacement));
        black_box(bits)
    });
}

fn bench_string(bencher: Bencher, len: usize, shape: ReplaceShape) {
    let input = make_string(len);
    let start = len / 3;
    let old_len = chunk_len(len);
    let replacement = make_string(replacement_len(old_len, shape));

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        string.replace_range(black_box(start..start + old_len), black_box(&replacement));
        black_box(string)
    });
}

#[inline]
fn chunk_len(len: usize) -> usize {
    (len / 8).max(1)
}

#[inline]
fn replacement_len(old_len: usize, shape: ReplaceShape) -> usize {
    match shape {
        ReplaceShape::SameLen => old_len,
        ReplaceShape::Shorter => (old_len / 2).max(1),
        ReplaceShape::Longer => old_len * 2,
    }
}

#[inline]
fn iv(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
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
