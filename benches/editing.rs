use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Pattern {
    Dense,
    Alternating,
}

macro_rules! pair_case {
    ($name:ident, $len:expr, $bit_fn:ident, $string_fn:ident) => {
        #[divan::bench_group]
        mod $name {
            use super::*;

            #[divan::bench]
            fn bit_string(bencher: Bencher) {
                $bit_fn(bencher, $len);
            }

            #[divan::bench]
            fn string(bencher: Bencher) {
                $string_fn(bencher, $len);
            }
        }
    };
}

macro_rules! len_cases {
    ($bit_fn:ident, $string_fn:ident) => {
        pair_case!(len_65, 65, $bit_fn, $string_fn);
        pair_case!(len_65536, 65_536, $bit_fn, $string_fn);
    };
}

#[divan::bench_group]
mod push {
    use super::*;
    len_cases!(bench_bit_string_push, bench_string_push);
}

#[divan::bench_group]
mod insert_front {
    use super::*;
    len_cases!(bench_bit_string_insert_front, bench_string_insert_front);
}

#[divan::bench_group]
mod insert_middle {
    use super::*;
    len_cases!(bench_bit_string_insert_middle, bench_string_insert_middle);
}

#[divan::bench_group]
mod remove_middle {
    use super::*;
    len_cases!(bench_bit_string_remove_middle, bench_string_remove_middle);
}

#[divan::bench_group]
mod push_bits {
    use super::*;
    len_cases!(bench_bit_string_push_bits, bench_string_push_str);
}

#[divan::bench_group]
mod insert_bits_middle {
    use super::*;
    len_cases!(
        bench_bit_string_insert_bits_middle,
        bench_string_insert_str_middle
    );
}

#[divan::bench_group]
mod split_off_middle {
    use super::*;
    len_cases!(
        bench_bit_string_split_off_middle,
        bench_string_split_off_middle
    );
}

#[divan::bench_group]
mod replace_same_len {
    use super::*;
    len_cases!(
        bench_bit_string_replace_same_len,
        bench_string_replace_same_len
    );
}

#[divan::bench_group]
mod replace_shorter {
    use super::*;
    len_cases!(
        bench_bit_string_replace_shorter,
        bench_string_replace_shorter
    );
}

#[divan::bench_group]
mod replace_longer {
    use super::*;
    len_cases!(bench_bit_string_replace_longer, bench_string_replace_longer);
}

#[divan::bench_group]
mod drain_interval {
    use super::*;
    len_cases!(bench_bit_string_drain_interval, bench_string_drain);
}

#[divan::bench_group]
mod slice {
    use super::*;
    len_cases!(bench_bit_string_slice, bench_string_slice_to_owned);
}

fn bench_bit_string_push(bencher: Bencher, len: usize) {
    let bits = make_bit_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.push(black_box(true));
            black_box(bits)
        });
}

fn bench_string_push(bencher: Bencher, len: usize) {
    let text = make_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.push(black_box('1'));
            black_box(text)
        });
}

fn bench_bit_string_insert_front(bencher: Bencher, len: usize) {
    let bits = make_bit_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.insert(black_box(0), black_box(true));
            black_box(bits)
        });
}

fn bench_string_insert_front(bencher: Bencher, len: usize) {
    let text = make_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.insert(black_box(0), black_box('1'));
            black_box(text)
        });
}

fn bench_bit_string_insert_middle(bencher: Bencher, len: usize) {
    let bits = make_bit_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.insert(black_box(mid), black_box(true));
            black_box(bits)
        });
}

fn bench_string_insert_middle(bencher: Bencher, len: usize) {
    let text = make_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.insert(black_box(mid), black_box('1'));
            black_box(text)
        });
}

fn bench_bit_string_remove_middle(bencher: Bencher, len: usize) {
    let bits = make_bit_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            let removed = bits.remove(black_box(mid));
            black_box((bits, removed))
        });
}

fn bench_string_remove_middle(bencher: Bencher, len: usize) {
    let text = make_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            let removed = text.remove(black_box(mid));
            black_box((text, removed))
        });
}

fn bench_bit_string_push_bits(bencher: Bencher, len: usize) {
    let rhs_len = chunk_len(len);
    let bits = make_bit_string(len, Pattern::Dense);
    let rhs = make_bit_string(rhs_len, Pattern::Alternating);

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.push_bits(black_box(&rhs));
            black_box(bits)
        });
}

fn bench_string_push_str(bencher: Bencher, len: usize) {
    let rhs_len = chunk_len(len);
    let text = make_string(len, Pattern::Dense);
    let rhs = make_string(rhs_len, Pattern::Alternating);

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.push_str(black_box(rhs.as_str()));
            black_box(text)
        });
}

fn bench_bit_string_insert_bits_middle(bencher: Bencher, len: usize) {
    let rhs_len = chunk_len(len);
    let bits = make_bit_string(len, Pattern::Dense);
    let rhs = make_bit_string(rhs_len, Pattern::Alternating);
    let mid = len / 2;

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.insert_bits(black_box(mid), black_box(&rhs));
            black_box(bits)
        });
}

fn bench_string_insert_str_middle(bencher: Bencher, len: usize) {
    let rhs_len = chunk_len(len);
    let text = make_string(len, Pattern::Dense);
    let rhs = make_string(rhs_len, Pattern::Alternating);
    let mid = len / 2;

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.insert_str(black_box(mid), black_box(rhs.as_str()));
            black_box(text)
        });
}

fn bench_bit_string_split_off_middle(bencher: Bencher, len: usize) {
    let bits = make_bit_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            let rhs = bits.split_off(black_box(mid));
            black_box((bits, rhs))
        });
}

fn bench_string_split_off_middle(bencher: Bencher, len: usize) {
    let text = make_string(len, Pattern::Dense);
    let mid = len / 2;

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            let rhs = text.split_off(black_box(mid));
            black_box((text, rhs))
        });
}

fn bench_bit_string_replace_same_len(bencher: Bencher, len: usize) {
    bench_bit_string_replace(bencher, len, ReplaceShape::SameLen);
}

fn bench_string_replace_same_len(bencher: Bencher, len: usize) {
    bench_string_replace(bencher, len, ReplaceShape::SameLen);
}

fn bench_bit_string_replace_shorter(bencher: Bencher, len: usize) {
    bench_bit_string_replace(bencher, len, ReplaceShape::Shorter);
}

fn bench_string_replace_shorter(bencher: Bencher, len: usize) {
    bench_string_replace(bencher, len, ReplaceShape::Shorter);
}

fn bench_bit_string_replace_longer(bencher: Bencher, len: usize) {
    bench_bit_string_replace(bencher, len, ReplaceShape::Longer);
}

fn bench_string_replace_longer(bencher: Bencher, len: usize) {
    bench_string_replace(bencher, len, ReplaceShape::Longer);
}

fn bench_bit_string_drain_interval(bencher: Bencher, len: usize) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let interval = iv(start, width);
    let bits = make_bit_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            let removed = bits.drain_interval(black_box(interval));
            black_box((bits, removed))
        });
}

fn bench_string_drain(bencher: Bencher, len: usize) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let end = start + width;
    let text = make_string(len, Pattern::Dense);

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            let removed = text.drain(black_box(start..end)).collect::<String>();
            black_box((text, removed))
        });
}

fn bench_bit_string_slice(bencher: Bencher, len: usize) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let interval = iv(start, width);
    let bits = make_bit_string(len, Pattern::Dense);

    bencher.bench(|| black_box(&bits).slice(black_box(interval)));
}

fn bench_string_slice_to_owned(bencher: Bencher, len: usize) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let end = start + width;
    let text = make_string(len, Pattern::Dense);

    bencher.bench(|| black_box(&text)[black_box(start..end)].to_owned());
}

#[derive(Clone, Copy)]
enum ReplaceShape {
    SameLen,
    Shorter,
    Longer,
}

fn bench_bit_string_replace(bencher: Bencher, len: usize, shape: ReplaceShape) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let interval = iv(start, width);

    let bits = make_bit_string(len, Pattern::Dense);
    let replacement = make_bit_string(replacement_len(width, shape), Pattern::Alternating);

    bencher
        .with_inputs(|| bits.clone())
        .bench_values(|mut bits| {
            bits.replace_interval(black_box(interval), black_box(&replacement));
            black_box(bits)
        });
}

fn bench_string_replace(bencher: Bencher, len: usize, shape: ReplaceShape) {
    let width = chunk_len(len);
    let start = (len - width) / 2;
    let end = start + width;

    let text = make_string(len, Pattern::Dense);
    let replacement = make_string(replacement_len(width, shape), Pattern::Alternating);

    bencher
        .with_inputs(|| text.clone())
        .bench_values(|mut text| {
            text.replace_range(black_box(start..end), black_box(replacement.as_str()));
            black_box(text)
        });
}

#[inline]
fn replacement_len(width: usize, shape: ReplaceShape) -> usize {
    match shape {
        ReplaceShape::SameLen => width,
        ReplaceShape::Shorter => (width / 2).max(1),
        ReplaceShape::Longer => width * 2,
    }
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
fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

fn make_string(len: usize, pattern: Pattern) -> String {
    (0..len)
        .map(|index| bit_char(bit_at(index, pattern)))
        .collect()
}

#[inline]
fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::Dense => mix64(index as u64) & 1 != 0,
        Pattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
