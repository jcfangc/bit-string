// benches/support/mod.rs

use bit_string::BitString;

/// Deterministic input shapes used by all benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Dense,
    Sparse,
    Alternating,
}

/// Matching benchmark fixture.
///
/// `haystack` contains `needle` at `position`.
#[derive(Debug, Clone)]
pub struct NeedleCase {
    pub position: usize,
    pub haystack_bits: BitString,
    pub needle_bits: BitString,
    pub haystack_string: String,
    pub needle_string: String,
}

#[inline]
pub fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

pub fn make_string(len: usize, pattern: Pattern) -> String {
    let mut out = String::with_capacity(len);

    for index in 0..len {
        out.push(bit_char(bit_at(index, pattern)));
    }

    out
}

pub fn make_needle(len: usize, position: usize, needle_len: usize) -> NeedleCase {
    assert!(
        position <= len,
        "needle position out of bounds: position={position}, len={len}",
    );

    assert!(
        needle_len <= len - position,
        "needle does not fit: position={position}, needle_len={needle_len}, len={len}",
    );

    let mut haystack = vec![false; len];
    let needle = make_needle_bits(needle_len);

    for (offset, &value) in needle.iter().enumerate() {
        haystack[position + offset] = value;
    }

    let haystack_string = bools_to_string(&haystack);
    let needle_string = bools_to_string(&needle);

    NeedleCase {
        position,
        haystack_bits: haystack.into_iter().collect(),
        needle_bits: needle.into_iter().collect(),
        haystack_string,
        needle_string,
    }
}

#[inline]
fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::Dense => mixed_bit(index),
        Pattern::Sparse => mix64(index as u64) & 63 == 0,
        Pattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn mixed_bit(index: usize) -> bool {
    mix64(index as u64) & 1 != 0
}

#[inline]
fn bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

fn bools_to_string(values: &[bool]) -> String {
    let mut out = String::with_capacity(values.len());

    for &value in values {
        out.push(bit_char(value));
    }

    out
}

fn make_needle_bits(len: usize) -> Vec<bool> {
    let mut out = Vec::with_capacity(len);

    for index in 0..len {
        out.push(mix64((index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15)) & 7 <= 2);
    }

    if len != 0 {
        out[0] = true;
        out[len - 1] = true;
    }

    out
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
