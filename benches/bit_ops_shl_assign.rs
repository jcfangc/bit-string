use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

// Shift by 1 bit (bit-level shift, no word-level shortcut).
#[divan::bench(name = "shl_assign/len_4096_by_1/owned")]
fn shl_len_4096_by_1_owned(bencher: Bencher) {
    bench_shl(bencher, 4096, 1);
}

#[divan::bench(name = "shl_assign/len_4096_by_1/assign")]
fn shl_len_4096_by_1_assign(bencher: Bencher) {
    bench_shl_assign(bencher, 4096, 1);
}

// Shift by 64 bits (word-level shift, the fast path).
#[divan::bench(name = "shl_assign/len_4096_by_64/owned")]
fn shl_len_4096_by_64_owned(bencher: Bencher) {
    bench_shl(bencher, 4096, 64);
}

#[divan::bench(name = "shl_assign/len_4096_by_64/assign")]
fn shl_len_4096_by_64_assign(bencher: Bencher) {
    bench_shl_assign(bencher, 4096, 64);
}

// Shift large array by 1 — worst-case for SIMD (cascading carries).
#[divan::bench(name = "shl_assign/len_65536_by_1/owned")]
fn shl_len_65536_by_1_owned(bencher: Bencher) {
    bench_shl(bencher, 65_536, 1);
}

#[divan::bench(name = "shl_assign/len_65536_by_1/assign")]
fn shl_len_65536_by_1_assign(bencher: Bencher) {
    bench_shl_assign(bencher, 65_536, 1);
}

// Shift by a mixed amount (both word and bit shift components).
#[divan::bench(name = "shl_assign/len_65536_by_17/owned")]
fn shl_len_65536_by_17_owned(bencher: Bencher) {
    bench_shl(bencher, 65_536, 17);
}

#[divan::bench(name = "shl_assign/len_65536_by_17/assign")]
fn shl_len_65536_by_17_assign(bencher: Bencher) {
    bench_shl_assign(bencher, 65_536, 17);
}

fn bench_shl(bencher: Bencher, len: usize, amount: usize) {
    let input = make_bit_string(len);

    bencher.bench(|| {
        let bits = black_box(input.clone());
        black_box(bits.shl(amount))
    });
}

fn bench_shl_assign(bencher: Bencher, len: usize, amount: usize) {
    let input = make_bit_string(len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.shl_assign(amount);
        black_box(bits)
    });
}

#[inline]
fn make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| bit_at(index)).collect()
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
