use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

fn owned(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    bencher.bench(|| black_box(black_box(&input).reverse_bits()));
}

fn assign(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|bits| {
        bits.reverse_bits_assign();
        black_box(&*bits);
    });
}

fn make_bit_string(len: usize) -> BitString {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state & 1 != 0
        })
        .collect()
}

macro_rules! define_case {
    ($case:ident, $len:literal, $label:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("reverse_bits/len_", $label, "/ours_owned"))]
            fn owned(bencher: Bencher) {
                super::owned(bencher, $len);
            }

            #[divan::bench(name = concat!("reverse_bits_assign/len_", $label, "/ours_assign"))]
            fn assign(bencher: Bencher) {
                super::assign(bencher, $len);
            }
        }
    };
}

define_case!(len_63, 63, "63");
define_case!(len_64, 64, "64");
define_case!(len_65, 65, "65");
define_case!(len_4096, 4_096, "4096");
define_case!(len_4097, 4_097, "4097");
define_case!(len_65536, 65_536, "65536");
define_case!(len_65537, 65_537, "65537");
