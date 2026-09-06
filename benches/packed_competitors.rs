#[path = "packed_support/mod.rs"]
mod support;

use compressed_intvec::fixed::UFixedVec;
use divan::{Bencher, black_box};
use grit_bitvec::{TypedBitElem, TypedBitVec};
use support::{LENGTHS, WIDTHS, codes, indices};
use sux::bits::BitFieldVec;
use value_traits::slices::{SliceByValue, SliceByValueMut};

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// compressed-intvec::FixedVec
// ---------------------------------------------------------------------------

#[divan::bench(name = "construct/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn construct_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        black_box(
            UFixedVec::<u8>::from_iter_builder(input.iter().copied(), usize::from(BITS))
                .build()
                .unwrap(),
        )
    });
}

#[divan::bench(name = "get/sequential/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn get_sequential_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap();
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "get/random/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn get_random_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    let indexes = indices(len);
    bencher.bench(|| {
        let value = black_box(&value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.get(index).unwrap();
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "iter/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn iter_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value.iter().fold(0u8, |checksum, code| checksum ^ code);
        black_box(checksum)
    });
}

#[divan::bench(name = "set/middle/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn set_middle_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    let index = len / 2;
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.set(index, 0);
        black_box(&*value);
    });
}

#[divan::bench(name = "push/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn push_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
    });
}

#[divan::bench(name = "pop/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn pop_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.pop());
        black_box(&*value);
    });
}

#[divan::bench(name = "extend/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn extend_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = fixed_vec(BITS, &codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend_from_slice(&extension);
        black_box(&*value);
    });
}

#[divan::bench(name = "insert/middle/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn insert_middle_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    let index = len / 2;
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.insert(index, 0);
        black_box(&*value);
    });
}

#[divan::bench(name = "remove/middle/fixed_vec", consts = WIDTHS, args = LENGTHS)]
fn remove_middle_fixed_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = fixed_vec(BITS, &input);
    let index = len / 2;
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.remove(index));
        black_box(&*value);
    });
}

// ---------------------------------------------------------------------------
// sux::BitFieldVec
// ---------------------------------------------------------------------------

#[divan::bench(name = "construct/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn construct_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let mut value = BitFieldVec::<Vec<usize>>::with_capacity(usize::from(BITS), len);
        value.extend(input.iter().map(|&code| usize::from(code)));
        black_box(value)
    });
}

#[divan::bench(name = "get/sequential/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn get_sequential_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.index_value(index) as u8;
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "get/random/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn get_random_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    let indexes = indices(len);
    bencher.bench(|| {
        let value = black_box(&value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.index_value(index) as u8;
        }
        black_box(checksum)
    });
}

#[divan::bench(name = "iter/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn iter_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value
            .iter()
            .fold(0u8, |checksum, code| checksum ^ code as u8);
        black_box(checksum)
    });
}

#[divan::bench(name = "set/middle/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn set_middle_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    let index = len / 2;
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.set_value(index, 0);
        black_box(&*value);
    });
}

#[divan::bench(name = "push/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn push_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
    });
}

#[divan::bench(name = "pop/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn pop_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = bit_field_vec(BITS, &input);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.pop());
        black_box(&*value);
    });
}

#[divan::bench(name = "extend/bit_field_vec", consts = WIDTHS, args = LENGTHS)]
fn extend_bit_field_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = bit_field_vec(BITS, &codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().map(|&code| usize::from(code)));
        black_box(&*value);
    });
}

// ---------------------------------------------------------------------------
// grit-bitvec::TypedBitVec
// ---------------------------------------------------------------------------

#[divan::bench(
    name = "construct/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn construct_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher.bench(|| black_box(grit_vec::<T>(&input)));
}

#[divan::bench(
    name = "get/sequential/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn get_sequential_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let value = grit_vec::<T>(&input);
    bencher.bench_local(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap();
        }
        black_box(checksum)
    });
}

#[divan::bench(
    name = "iter/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn iter_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let value = grit_vec::<T>(&input);
    bencher.bench_local(|| {
        let value = black_box(&value);
        let checksum = (0..len).fold(0u8, |checksum, index| checksum ^ value.get(index).unwrap());
        black_box(checksum)
    });
}

#[divan::bench(
    name = "set/middle/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn set_middle_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let index = len / 2;
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.set(index, 0));
            black_box(&*value);
        });
}

#[divan::bench(
    name = "push/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn push_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.push(0));
            black_box(&*value);
        });
}

#[divan::bench(
    name = "pop/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn pop_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.pop());
            black_box(&*value);
        });
}

#[divan::bench(
    name = "insert/middle/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn insert_middle_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let index = len / 2;
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.insert(index, 0));
            black_box(&*value);
        });
}

#[divan::bench(
    name = "remove/middle/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn remove_middle_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let index = len / 2;
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.remove(index));
            black_box(&*value);
        });
}

#[divan::bench(
    name = "append/typed_bit_vec",
    types = [
        grit_bitvec::u8_as_u1,
        grit_bitvec::u8_as_u2,
        grit_bitvec::u8_as_u3,
        grit_bitvec::u8_as_u4,
        grit_bitvec::u8_as_u5,
        grit_bitvec::u8_as_u6,
        grit_bitvec::u8_as_u7,
    ],
    args = LENGTHS,
)]
fn append_typed_bit_vec<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let extension = codes(grit_width::<T>(), len / 4);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.append_iter(extension.iter().copied()));
            black_box(&*value);
        });
}

fn fixed_vec(bits: u8, input: &[u8]) -> UFixedVec<u8> {
    UFixedVec::from_iter_builder(input.iter().copied(), usize::from(bits))
        .build()
        .unwrap()
}

fn bit_field_vec(bits: u8, input: &[u8]) -> BitFieldVec<Vec<usize>> {
    let mut value = BitFieldVec::with_capacity(usize::from(bits), input.len());
    value.extend(input.iter().map(|&code| usize::from(code)));
    value
}

fn grit_vec<T>(input: &[u8]) -> TypedBitVec<T>
where
    T: TypedBitElem<Base = u8>,
{
    let mut value = TypedBitVec::with_capacity(input.len());
    value.append_iter(input.iter().copied()).unwrap();
    value
}

fn grit_width<T>() -> u8
where
    T: TypedBitElem<Base = u8> + 'static,
{
    use std::any::TypeId;

    if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u1>() {
        1
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u2>() {
        2
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u3>() {
        3
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u4>() {
        4
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u5>() {
        5
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u6>() {
        6
    } else if TypeId::of::<T>() == TypeId::of::<grit_bitvec::u8_as_u7>() {
        7
    } else {
        unreachable!("unsupported grit-bitvec benchmark type")
    }
}
