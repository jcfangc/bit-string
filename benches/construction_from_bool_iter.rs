use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "from_bool_iter/len_65/bit_string")]
fn from_bool_iter_len_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65);
}

#[divan::bench(name = "from_bool_iter/len_65/bitvec_simd")]
fn from_bool_iter_len_65_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65);
}

#[divan::bench(name = "from_bool_iter/len_4096/bit_string")]
fn from_bool_iter_len_4096_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096);
}

#[divan::bench(name = "from_bool_iter/len_4096/bitvec_simd")]
fn from_bool_iter_len_4096_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 4096);
}

#[divan::bench(name = "from_bool_iter/len_65536/bit_string")]
fn from_bool_iter_len_65536_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65536);
}

#[divan::bench(name = "from_bool_iter/len_65536/bitvec_simd")]
fn from_bool_iter_len_65536_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65536);
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let input: Vec<bool> = (0..len).map(|i| bit_at(i)).collect();
    bencher.bench(|| black_box(input.iter().copied().collect::<BitString>()));
}

fn bench_bitvec_simd(bencher: Bencher, len: usize) {
    let input: Vec<bool> = (0..len).map(|i| bit_at(i)).collect();
    bencher.bench(|| black_box(BitVec::from_bool_iterator(input.iter().copied())));
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
