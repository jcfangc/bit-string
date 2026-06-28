use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "shl/len_65/amount_1/ours")]
fn shl_len_65_amount_1_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65, 1);
}

#[divan::bench(name = "shl/len_4096/amount_1/ours")]
fn shl_len_4096_amount_1_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096, 1);
}

#[divan::bench(name = "shl/len_4096/amount_65/ours")]
fn shl_len_4096_amount_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096, 65);
}

#[divan::bench(name = "shl/len_65536/amount_1/ours")]
fn shl_len_65536_amount_1_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536, 1);
}

#[divan::bench(name = "shl/len_65536/amount_65/ours")]
fn shl_len_65536_amount_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536, 65);
}

fn bench_bit_string(bencher: Bencher, len: usize, amount: usize) {
    let bits = make_bit_string(len);

    bencher.bench(|| black_box(&bits).shl(black_box(amount)));
}

#[inline]
fn make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| mix64(index as u64) & 1 != 0).collect()
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
