use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

fn make_bits(len: usize) -> BitString {
    (0..len).map(|index| index % 7 == 1).collect()
}

fn reference(bits: impl Iterator<Item = bool>) -> String {
    bits.map(|bit| if bit { '1' } else { '0' }).collect()
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let bits = make_bits(len);
    assert_eq!(bits.to_string(), reference(bits.iter()));

    bencher.bench(|| black_box(black_box(&bits).to_string()));
}

fn bench_aligned_bit_str(bencher: Bencher, len: usize) {
    let owner = make_bits(len);
    let bits = owner.as_bit_str();
    assert_eq!(bits.to_string(), reference(bits.iter()));

    bencher.bench(|| black_box(black_box(bits).to_string()));
}

fn bench_unaligned_bit_str(bencher: Bencher, len: usize) {
    let owner = make_bits(len + 1);
    let bits = owner.as_bit_str().slice_from(1);
    assert_eq!(bits.to_string(), reference(bits.iter()));

    bencher.bench(|| black_box(black_box(bits).to_string()));
}

macro_rules! define_case {
    ($name:ident, $len:literal, $label:literal) => {
        mod $name {
            use super::*;

            #[divan::bench(name = concat!("display/len_", $label, "/ours_bit_string"))]
            fn bit_string(bencher: Bencher) {
                bench_bit_string(bencher, $len);
            }

            #[divan::bench(name = concat!("display/len_", $label, "/ours_bit_str_aligned"))]
            fn bit_str_aligned(bencher: Bencher) {
                bench_aligned_bit_str(bencher, $len);
            }

            #[divan::bench(name = concat!("display/len_", $label, "/ours_bit_str_unaligned"))]
            fn bit_str_unaligned(bencher: Bencher) {
                bench_unaligned_bit_str(bencher, $len);
            }
        }
    };
}

define_case!(len_64, 64, "64");
define_case!(len_4096, 4_096, "4096");
define_case!(len_65536, 65_536, "65536");
