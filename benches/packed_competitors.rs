#[path = "packed_support/mod.rs"]
mod support;

use compressed_intvec::fixed::UFixedVec;
use divan::{Bencher, black_box};
use grit_bitvec::{TypedBitElem, TypedBitVec};
use support::{codes, indices};
use sux::bits::BitFieldVec;
use value_traits::slices::{SliceByValue, SliceByValueMut};

fn main() {
    divan::main();
}

fn construct_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        black_box(
            UFixedVec::<u8>::from_iter_builder(input.iter().copied(), usize::from(BITS))
                .build()
                .unwrap(),
        )
    });
}

fn get_sequential_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap();
        }
        black_box(checksum)
    });
}

fn get_random_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
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

fn iter_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value.iter().fold(0u8, |checksum, code| checksum ^ code);
        black_box(checksum)
    });
}

fn set_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.set(len / 2, 0);
        black_box(&*value);
    });
}

fn push_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
    });
}

fn pop_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.pop());
        black_box(&*value);
    });
}

fn extend_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.extend_from_slice(&extension);
        black_box(&*value);
    });
}

fn insert_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.insert(len / 2, 0);
        black_box(&*value);
    });
}

fn remove_fixed<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = fixed_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.remove(len / 2));
        black_box(&*value);
    });
}

fn construct_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let mut value = BitFieldVec::<Vec<usize>>::with_capacity(usize::from(BITS), len);
        value.extend(input.iter().map(|&code| usize::from(code)));
        black_box(value)
    });
}

fn get_sequential_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.index_value(index) as u8;
        }
        black_box(checksum)
    });
}

fn get_random_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
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

fn iter_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value
            .iter()
            .fold(0u8, |checksum, code| checksum ^ code as u8);
        black_box(checksum)
    });
}

fn set_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.set_value(len / 2, 0);
        black_box(&*value);
    });
}

fn push_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
    });
}

fn pop_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        black_box(value.pop());
        black_box(&*value);
    });
}

fn extend_sux<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = sux_vec(BITS, &codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| value.clone()).bench_refs(|value| {
        value.extend(extension.iter().map(|&code| usize::from(code)));
        black_box(&*value);
    });
}

fn construct_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher.bench(|| black_box(grit_vec::<T>(&input)));
}

fn get_sequential_grit<T>(bencher: Bencher, len: usize)
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

fn get_random_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    let value = grit_vec::<T>(&input);
    let indexes = indices(len);
    bencher.bench_local(|| {
        let value = black_box(&value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.get(index).unwrap();
        }
        black_box(checksum)
    });
}

fn iter_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_values(|value| {
            let checksum = value
                .into_iter()
                .fold(0u8, |checksum, code| checksum ^ code);
            black_box(checksum)
        });
}

fn set_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.set(len / 2, 0));
            black_box(&*value);
        });
}

fn push_grit<T>(bencher: Bencher, len: usize)
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

fn pop_grit<T>(bencher: Bencher, len: usize)
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

fn insert_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.insert(len / 2, 0));
            black_box(&*value);
        });
}

fn remove_grit<T>(bencher: Bencher, len: usize)
where
    T: TypedBitElem<Base = u8> + 'static,
{
    let input = codes(grit_width::<T>(), len);
    bencher
        .with_inputs(|| grit_vec::<T>(&input))
        .bench_local_refs(|value| {
            let _ = black_box(value.remove(len / 2));
            black_box(&*value);
        });
}

fn append_grit<T>(bencher: Bencher, len: usize)
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

macro_rules! define_packed_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! benchmark {
                        ($scenario:literal, $helper:ident, $legend:literal) => {
                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/",
                                                                            $legend
                                                                        )
                                                                    )]
                            fn $helper(bencher: Bencher) {
                                super::$helper::<$bits>(bencher, $len);
                            }
                        };
                    }

            benchmark!(
                "construction/from_chars",
                construct_fixed,
                "compressed_intvec"
            );
            benchmark!(
                "access/get_sequential",
                get_sequential_fixed,
                "compressed_intvec"
            );
            benchmark!("access/get_random", get_random_fixed, "compressed_intvec");
            benchmark!("iteration/iterate", iter_fixed, "compressed_intvec");
            benchmark!("editing/set_middle", set_fixed, "compressed_intvec");
            benchmark!("editing/push", push_fixed, "compressed_intvec");
            benchmark!("editing/pop", pop_fixed, "compressed_intvec");
            benchmark!("editing/extend", extend_fixed, "compressed_intvec");
            benchmark!("editing/insert_middle", insert_fixed, "compressed_intvec");
            benchmark!("editing/remove_middle", remove_fixed, "compressed_intvec");

            benchmark!("construction/from_chars", construct_sux, "sux");
            benchmark!("access/get_sequential", get_sequential_sux, "sux");
            benchmark!("access/get_random", get_random_sux, "sux");
            benchmark!("iteration/iterate", iter_sux, "sux");
            benchmark!("editing/set_middle", set_sux, "sux");
            benchmark!("editing/push", push_sux, "sux");
            benchmark!("editing/pop", pop_sux, "sux");
            benchmark!("editing/extend", extend_sux, "sux");
        }
    };
}

crate::for_each_packed_case!(define_packed_case);

macro_rules! define_grit_case {
    ($case:ident, $bits:literal, $len:literal, $type:ty) => {
        mod $case {
            use super::*;

            macro_rules! benchmark {
                        ($scenario:literal, $helper:ident) => {
                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/grit_bitvec"
                                                                        )
                                                                    )]
                            fn $helper(bencher: Bencher) {
                                crate::$helper::<$type>(bencher, $len);
                            }
                        };
                    }

            benchmark!("construction/from_chars", construct_grit);
            benchmark!("access/get_sequential", get_sequential_grit);
            benchmark!("access/get_random", get_random_grit);
            benchmark!("iteration/iterate", iter_grit);
            benchmark!("editing/set_middle", set_grit);
            benchmark!("editing/push", push_grit);
            benchmark!("editing/pop", pop_grit);
            benchmark!("editing/insert_middle", insert_grit);
            benchmark!("editing/remove_middle", remove_grit);
            benchmark!("editing/extend", append_grit);
        }
    };
}

mod grit_cases {
    use super::*;
    crate::for_each_grit_case!(define_grit_case);
}

fn fixed_vec(bits: u8, input: &[u8]) -> UFixedVec<u8> {
    UFixedVec::from_iter_builder(input.iter().copied(), usize::from(bits))
        .build()
        .unwrap()
}

fn sux_vec(bits: u8, input: &[u8]) -> BitFieldVec<Vec<usize>> {
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
