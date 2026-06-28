use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "zeros/len_65/ours")]
fn zeros_len_65_bit_string(bencher: Bencher) {
    bencher.bench(|| black_box(BitString::zeros(65)));
}

#[divan::bench(name = "zeros/len_65/bitvec_simd")]
fn zeros_len_65_bitvec_simd(bencher: Bencher) {
    bencher.bench(|| black_box(BitVec::zeros(65)));
}

#[divan::bench(name = "zeros/len_4096/ours")]
fn zeros_len_4096_bit_string(bencher: Bencher) {
    bencher.bench(|| black_box(BitString::zeros(4096)));
}

#[divan::bench(name = "zeros/len_4096/bitvec_simd")]
fn zeros_len_4096_bitvec_simd(bencher: Bencher) {
    bencher.bench(|| black_box(BitVec::zeros(4096)));
}

#[divan::bench(name = "zeros/len_65536/ours")]
fn zeros_len_65536_bit_string(bencher: Bencher) {
    bencher.bench(|| black_box(BitString::zeros(65_536)));
}

#[divan::bench(name = "zeros/len_65536/bitvec_simd")]
fn zeros_len_65536_bitvec_simd(bencher: Bencher) {
    bencher.bench(|| black_box(BitVec::zeros(65_536)));
}
