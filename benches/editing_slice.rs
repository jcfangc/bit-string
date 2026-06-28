use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

// Flattened slice benchmarks for CodSpeed compatibility
#[divan::bench(name = "slice/len_65/ours_string")]
fn slice_len_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65);
}

#[divan::bench(name = "slice/len_65/string")]
fn slice_len_65_string(bencher: Bencher) {
    bench_string(bencher, 65);
}

#[divan::bench(name = "slice/len_65536/ours_string")]
fn slice_len_65536_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "slice/len_65536/string")]
fn slice_len_65536_string(bencher: Bencher) {
    bench_string(bencher, 65_536);
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let start = len / 3;
    let slice_len = chunk_len(len);
    let interval = iv(start, slice_len);

    bencher.bench(|| black_box(&input).slice(black_box(interval)));
}

fn bench_string(bencher: Bencher, len: usize) {
    let input = make_string(len);
    let start = len / 3;
    let slice_len = chunk_len(len);

    bencher.bench(|| black_box(&input[start..start + slice_len]).to_owned());
}

#[inline]
fn chunk_len(len: usize) -> usize {
    (len / 8).max(1)
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
