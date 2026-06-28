use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum P {
    Z,
    A,
    D,
}

#[divan::bench(name = "trailing_zeros/len_65/all_zeros/ours_str")]
fn t65z_v(b: Bencher) {
    let bits = bs(65, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65/all_zeros/ours_string")]
fn t65z_o(b: Bencher) {
    let bits = bs(65, P::Z);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65/alternating/ours_str")]
fn t65a_v(b: Bencher) {
    let bits = bs(65, P::A);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65/alternating/ours_string")]
fn t65a_o(b: Bencher) {
    let bits = bs(65, P::A);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65/dense/ours_str")]
fn t65d_v(b: Bencher) {
    let bits = bs(65, P::D);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65/dense/ours_string")]
fn t65d_o(b: Bencher) {
    let bits = bs(65, P::D);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_4096/all_zeros/ours_str")]
fn t4z_v(b: Bencher) {
    let bits = bs(4096, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_4096/all_zeros/ours_string")]
fn t4z_o(b: Bencher) {
    let bits = bs(4096, P::Z);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_4096/dense/ours_str")]
fn t4d_v(b: Bencher) {
    let bits = bs(4096, P::D);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_4096/dense/ours_string")]
fn t4d_o(b: Bencher) {
    let bits = bs(4096, P::D);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65536/all_zeros/ours_str")]
fn t6z_v(b: Bencher) {
    let bits = bs(65536, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65536/all_zeros/ours_string")]
fn t6z_o(b: Bencher) {
    let bits = bs(65536, P::Z);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65536/dense/ours_str")]
fn t6d_v(b: Bencher) {
    let bits = bs(65536, P::D);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/len_65536/dense/ours_string")]
fn t6d_o(b: Bencher) {
    let bits = bs(65536, P::D);
    b.bench(|| black_box(&bits).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/unaligned_3/len_4096/all_zeros/ours_str")]
fn tu3_v(b: Bencher) {
    let bits = bs_unaligned(4096, 3, P::Z);
    let sub = slice_sub(&bits, 3, 4096);
    b.bench(|| black_box(&sub).trailing_zeros());
}
#[divan::bench(name = "trailing_zeros/unaligned_63/len_4096/all_zeros/ours_str")]
fn tu63_v(b: Bencher) {
    let bits = bs_unaligned(4096, 63, P::Z);
    let sub = slice_sub(&bits, 63, 4096);
    b.bench(|| black_box(&sub).trailing_zeros());
}

fn bs(len: usize, p: P) -> BitString {
    (0..len).map(|i| bit(i, p)).collect()
}
fn bs_unaligned(len: usize, skip: usize, p: P) -> BitString {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, bit(i - skip, p));
    }
    bits
}
fn slice_sub(bits: &BitString, skip: usize, len: usize) -> bit_string::BitStr<'_> {
    bits.as_bit_str()
        .slice(UsizeCO::try_new(skip, skip + len).unwrap())
}
fn bit(i: usize, p: P) -> bool {
    match p {
        P::Z => false,
        P::A => i % 2 != 0,
        P::D => mix64(i as u64) & 1 != 0,
    }
}
fn mix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e37_79b9_7f4a_7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}
