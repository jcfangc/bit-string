use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "push_bit_string/len_65/ours")]
fn push_bits_len_65_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65);
}

#[divan::bench(name = "push_bit_string/len_65/string")]
fn push_bits_len_65_string(bencher: Bencher) {
    bench_string(bencher, 65);
}

#[divan::bench(name = "push_bit_string/len_65536/ours")]
fn push_bits_len_65536_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "push_bit_string/len_65536/string")]
fn push_bits_len_65536_string(bencher: Bencher) {
    bench_string(bencher, 65_536);
}

fn bench_bit_string(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let rhs = make_bit_string(chunk_len(len));

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.push_bit_string(black_box(&rhs));
        black_box(bits)
    });
}

fn bench_string(bencher: Bencher, len: usize) {
    let input = make_string(len);
    let rhs = make_string(chunk_len(len));

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        string.push_str(black_box(&rhs));
        black_box(string)
    });
}

#[inline]
fn chunk_len(len: usize) -> usize {
    (len / 8).max(1)
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
