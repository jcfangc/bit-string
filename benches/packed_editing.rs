#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{Code, LENGTHS, WIDTHS, codes, packed};

fn main() {
    divan::main();
}

#[divan::bench(name = "set/middle/packed", consts = WIDTHS, args = LENGTHS)]
fn set_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = len / 2;
    bencher
        .with_inputs(|| input.clone())
        .bench_refs(|value| black_box(value.set(index, Code(0))));
}

#[divan::bench(name = "set/middle/vec", consts = WIDTHS, args = LENGTHS)]
fn set_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let old = std::mem::replace(&mut value[index], 0);
        black_box(old)
    });
}

#[divan::bench(name = "push/packed", consts = WIDTHS, args = LENGTHS)]
fn push_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(Code(0));
        black_box(value.char_len())
    });
}

#[divan::bench(name = "push/vec", consts = WIDTHS, args = LENGTHS)]
fn push_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(0);
        black_box(value.len())
    });
}

#[divan::bench(name = "pop/packed", consts = WIDTHS, args = LENGTHS)]
fn pop_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher
        .with_inputs(|| input.clone())
        .bench_refs(|value| black_box(value.pop()));
}

#[divan::bench(name = "pop/vec", consts = WIDTHS, args = LENGTHS)]
fn pop_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher
        .with_inputs(|| input.clone())
        .bench_refs(|value| black_box(value.pop()));
}

#[divan::bench(name = "insert/middle/packed", consts = WIDTHS, args = LENGTHS)]
fn insert_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, Code(0));
        black_box(value.char_len())
    });
}

#[divan::bench(name = "insert/middle/vec", consts = WIDTHS, args = LENGTHS)]
fn insert_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, 0);
        black_box(value.len())
    });
}

#[divan::bench(name = "remove/middle/packed", consts = WIDTHS, args = LENGTHS)]
fn remove_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = len / 2;
    bencher
        .with_inputs(|| input.clone())
        .bench_refs(|value| black_box(value.remove(index)));
}

#[divan::bench(name = "remove/middle/vec", consts = WIDTHS, args = LENGTHS)]
fn remove_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
    bencher
        .with_inputs(|| input.clone())
        .bench_refs(|value| black_box(value.remove(index)));
}

#[divan::bench(name = "extend/packed", consts = WIDTHS, args = LENGTHS)]
fn extend_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied().map(Code));
        black_box(value.char_len())
    });
}

#[divan::bench(name = "extend/vec", consts = WIDTHS, args = LENGTHS)]
fn extend_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied());
        black_box(value.len())
    });
}
