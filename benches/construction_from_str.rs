use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "from_str/len_65/bit_string")]
fn from_str_len_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65);
}

#[divan::bench(name = "from_str/len_4096/bit_string")]
fn from_str_len_4096_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096);
}

#[divan::bench(name = "from_str/len_65536/bit_string")]
fn from_str_len_65536_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536);
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let input = make_str(len);
    bencher.bench(|| black_box(BitString::try_from(black_box(input.as_str())).unwrap()));
}

#[inline]
fn make_str(len: usize) -> String {
    let mut s = String::with_capacity(len);
    for i in 0..len {
        s.push(bit_char(bit_at(i)));
    }
    s
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
