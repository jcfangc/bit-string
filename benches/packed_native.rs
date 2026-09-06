#[path = "packed_support/mod.rs"]
mod support;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use divan::{Bencher, black_box};
use int_intervals::UsizeCO;
use support::{LENGTHS, WIDTHS, codes, packed};

const SEARCH_LENGTHS: &[usize] = &[64, 1_024, 65_536, 1_048_576];
const NEEDLE_LENGTH: usize = 8;

fn main() {
    divan::main();
}

// These operations have no common competitor API in the external baseline.
// They establish the semantic PackedString/PackedStr baseline for later work.

#[divan::bench(name = "matching/find/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn find_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    bencher.bench(|| black_box(value.find(&needle)));
}

#[divan::bench(name = "matching/find/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn find_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    let value = value.as_packed_str();
    let needle = needle.as_packed_str();
    bencher.bench(|| black_box(value.find(needle)));
}

#[divan::bench(name = "matching/rfind/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn rfind_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    bencher.bench(|| black_box(value.rfind(&needle)));
}

#[divan::bench(name = "matching/rfind/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn rfind_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    let value = value.as_packed_str();
    let needle = needle.as_packed_str();
    bencher.bench(|| black_box(value.rfind(needle)));
}

#[divan::bench(name = "matching/contains/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn contains_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    bencher.bench(|| black_box(value.contains(&needle)));
}

#[divan::bench(name = "matching/contains/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn contains_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&input[..NEEDLE_LENGTH.min(len)]);
    let value = value.as_packed_str();
    let needle = needle.as_packed_str();
    bencher.bench(|| black_box(value.contains(needle)));
}

#[divan::bench(name = "matching/matches_at/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn matches_at_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.matches_at(start, &needle)));
}

#[divan::bench(name = "matching/matches_at/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn matches_at_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    let value = value.as_packed_str();
    let needle = needle.as_packed_str();
    bencher.bench(|| black_box(value.matches_at(start, needle)));
}

#[divan::bench(name = "matching/starts_with/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn starts_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.starts_with(&prefix)));
}

#[divan::bench(name = "matching/starts_with/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn starts_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value.as_packed_str();
    let prefix = prefix.as_packed_str();
    bencher.bench(|| black_box(value.starts_with(prefix)));
}

#[divan::bench(name = "matching/ends_with/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn ends_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(value.ends_with(&suffix)));
}

#[divan::bench(name = "matching/ends_with/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn ends_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value.as_packed_str();
    let suffix = suffix.as_packed_str();
    bencher.bench(|| black_box(value.ends_with(suffix)));
}

#[divan::bench(name = "matching/strip_prefix/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn strip_prefix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.strip_prefix(&prefix)));
}

#[divan::bench(name = "matching/strip_prefix/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn strip_prefix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value.as_packed_str();
    let prefix = prefix.as_packed_str();
    bencher.bench(|| black_box(value.strip_prefix(prefix)));
}

#[divan::bench(name = "matching/strip_suffix/packed_string", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn strip_suffix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(value.strip_suffix(&suffix)));
}

#[divan::bench(name = "matching/strip_suffix/packed_str", consts = WIDTHS, args = SEARCH_LENGTHS)]
fn strip_suffix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value.as_packed_str();
    let suffix = suffix.as_packed_str();
    bencher.bench(|| black_box(value.strip_suffix(suffix)));
}

#[divan::bench(name = "ordering/cmp/packed_string", consts = WIDTHS, args = LENGTHS)]
fn cmp_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let left = packed::<BITS>(&codes(BITS, len));
    let right = packed::<BITS>(&codes(BITS, len).into_iter().rev().collect::<Vec<_>>());
    bencher.bench(|| black_box(left.cmp(&right)));
}

#[divan::bench(name = "ordering/cmp/packed_str", consts = WIDTHS, args = LENGTHS)]
fn cmp_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_owner = packed::<BITS>(&codes(BITS, len));
    let right_owner = packed::<BITS>(&codes(BITS, len).into_iter().rev().collect::<Vec<_>>());
    let left = left_owner.as_packed_str();
    let right = right_owner.as_packed_str();
    bencher.bench(|| black_box(left.cmp(&right)));
}

#[divan::bench(name = "hash/packed_string", consts = WIDTHS, args = LENGTHS)]
fn hash_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    bencher.bench(|| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        black_box(hasher.finish())
    });
}

#[divan::bench(name = "slice/packed_string", consts = WIDTHS, args = LENGTHS)]
fn slice_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    let interval = interval(len / 4, len / 2);
    bencher.bench(|| black_box(value.slice(interval)));
}

#[divan::bench(name = "slice/packed_str", consts = WIDTHS, args = LENGTHS)]
fn slice_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let owner = packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str();
    let interval = interval(len / 4, len / 2);
    bencher.bench(|| black_box(value.slice(interval)));
}

#[divan::bench(name = "conversion/to_packed_string", consts = WIDTHS, args = LENGTHS)]
fn to_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let owner = packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str();
    bencher.bench(|| black_box(value.to_packed_string()));
}

#[divan::bench(name = "editing/replace_interval", consts = WIDTHS, args = LENGTHS)]
fn replace_interval<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    let replacement = packed::<BITS>(&codes(BITS, len / 4));
    let interval = interval(len / 4, len / 4);
    bencher.bench(|| black_box(value.replace_interval(interval, &replacement)));
}

fn interval(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
}
