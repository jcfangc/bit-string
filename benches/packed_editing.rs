#[path = "packed_support/mod.rs"]
mod support;

use bit_string::PackedString;
use divan::{Bencher, black_box};
use support::{Code, aligned_index, codes, cross_word_index, packed, unaligned_index};

fn main() {
    divan::main();
}

fn set_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input_codes = codes(BITS, len);
    let index = aligned_index::<BITS>(len);
    let replacement = Code(input_codes[index] ^ 1);
    let input = packed::<BITS>(&input_codes);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let previous = value.set(index, replacement);
        black_box(&*value);
        black_box(previous)
    });
}

fn set_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = aligned_index::<BITS>(len);
    let replacement = input[index] ^ 1;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let old = std::mem::replace(&mut value[index], replacement);
        black_box(&*value);
        black_box(old)
    });
}

fn push_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    push_packed_value::<BITS>(bencher, len, 0);
}

fn push_packed_value<const BITS: u8>(bencher: Bencher, len: usize, code: u8) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(Code(code));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn push_amortized_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    push_amortized_packed_value::<BITS>(bencher, len, 0);
}

fn push_amortized_packed_value<const BITS: u8>(bencher: Bencher, len: usize, code: u8) {
    let input = packed::<BITS>(&codes(BITS, len - 1));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(Code(code));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn push_existing_word<const BITS: u8>(bencher: Bencher, code: u8) {
    push_packed_value::<BITS>(bencher, 1, code);
}

// Build the backing word first, then truncate so the timed push grows the
// logical word slice without triggering a heap reallocation.
fn push_word_growth<const BITS: u8>(bencher: Bencher, code: u8) {
    const PREFIX_LEN: usize = 64;
    let input = packed::<BITS>(&codes(BITS, PREFIX_LEN + 1));
    bencher
        .with_inputs(|| {
            let mut value = input.clone();
            value.truncate(PREFIX_LEN);
            value
        })
        .bench_refs(|value| {
            value.push(Code(code));
            black_box(&*value);
            black_box(value.char_len())
        });
}

fn push_cross_word<const BITS: u8>(bencher: Bencher, code: u8) {
    let prefix_len = match BITS {
        3 => 21,
        5 => 12,
        6 => 10,
        7 => 9,
        _ => unreachable!("cross-word character only exists for irregular widths"),
    };
    let input = packed::<BITS>(&codes(BITS, prefix_len + 1));
    bencher
        .with_inputs(|| {
            let mut value = input.clone();
            value.truncate(prefix_len);
            value
        })
        .bench_refs(|value| {
            value.push(Code(code));
            black_box(&*value);
            black_box(value.char_len())
        });
}

fn max_code(bits: u8) -> u8 {
    if bits == 8 {
        u8::MAX
    } else {
        ((1u16 << bits) - 1) as u8
    }
}

fn push_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
        black_box(value.len())
    });
}

fn push_existing_word_vec(bencher: Bencher, code: u8) {
    bencher
        .with_inputs(|| {
            let mut value = vec![0u8];
            value.reserve(1);
            value
        })
        .bench_refs(|value| {
            value.push(code);
            black_box(&*value);
            black_box(value.len())
        });
}

fn push_word_growth_vec(bencher: Bencher, code: u8) {
    let input = vec![0u8; 65];
    bencher
        .with_inputs(|| {
            let mut value = input.clone();
            value.truncate(64);
            value
        })
        .bench_refs(|value| {
            value.push(code);
            black_box(&*value);
            black_box(value.len())
        });
}

fn push_amortized_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len - 1);
    bencher
        .with_inputs(|| {
            let mut value = input.clone();
            value.reserve(1);
            value
        })
        .bench_refs(|value| {
            value.push(0);
            black_box(&*value);
            black_box(value.len())
        });
}

fn pop_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let popped = value.pop();
        black_box(&*value);
        black_box(popped)
    });
}

fn pop_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let popped = value.pop();
        black_box(&*value);
        black_box(popped)
    });
}

fn insert_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = unaligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, Code(0));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn insert_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = unaligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, 0);
        black_box(&*value);
        black_box(value.len())
    });
}

fn remove_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = cross_word_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let removed = value.remove(index);
        black_box(&*value);
        black_box(removed)
    });
}

fn remove_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = cross_word_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let removed = value.remove(index);
        black_box(&*value);
        black_box(removed)
    });
}

fn extend_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    extend_packed_at::<BITS>(bencher, len, len);
}

fn extend_packed_unaligned<const BITS: u8>(bencher: Bencher, len: usize) {
    extend_packed_at::<BITS>(bencher, len, len + 1);
}

fn extend_packed_at<const BITS: u8>(bencher: Bencher, len: usize, prefix_len: usize) {
    let input = packed::<BITS>(&codes(BITS, prefix_len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied().map(Code));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn extend_from_chars_concat<const BITS: u8>(bencher: Bencher, len: usize) {
    extend_from_chars_concat_at::<BITS>(bencher, len, len);
}

fn extend_from_chars_concat_unaligned<const BITS: u8>(bencher: Bencher, len: usize) {
    extend_from_chars_concat_at::<BITS>(bencher, len, len + 1);
}

fn extend_from_chars_concat_at<const BITS: u8>(bencher: Bencher, len: usize, prefix_len: usize) {
    let input = packed::<BITS>(&codes(BITS, prefix_len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let suffix = packed::<BITS>(&extension);
        value.push_packed_string(&suffix);
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn extend_with_length<const BITS: u8>(bencher: Bencher, extension_len: usize, concat: bool) {
    let input = extend_length_input::<BITS>(extension_len);
    let extension = codes(BITS, extension_len);
    bencher
        .with_inputs(|| extend_length_bench_input(&input))
        .bench_refs(|value| {
            if concat {
                let suffix = packed::<BITS>(&extension);
                value.push_packed_string(&suffix);
            } else {
                value.extend(extension.iter().copied().map(Code));
            }
            black_box(&*value);
            black_box(value.char_len())
        });
}

fn extend_per_character_with_length<const BITS: u8>(bencher: Bencher, extension_len: usize) {
    let input = extend_length_input::<BITS>(extension_len);
    let extension = codes(BITS, extension_len);
    bencher
        .with_inputs(|| extend_length_bench_input(&input))
        .bench_refs(|value| {
            for code in extension.iter().copied() {
                value.push(Code(code));
            }
            black_box(&*value);
            black_box(value.char_len())
        });
}

fn extend_length_input<const BITS: u8>(extension_len: usize) -> PackedString<Code, BITS> {
    const PREFIX_LEN: usize = 1_048_576;
    packed::<BITS>(&codes(BITS, PREFIX_LEN + extension_len))
}

fn extend_length_bench_input<const BITS: u8>(
    input: &PackedString<Code, BITS>,
) -> PackedString<Code, BITS> {
    const PREFIX_LEN: usize = 1_048_576;
    // The source was built with spare logical words. Clone before truncating so
    // every candidate gets the same retained capacity outside the timed region.
    let mut value = input.clone();
    value.truncate(PREFIX_LEN);
    value
}

fn extend_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied());
        black_box(&*value);
        black_box(value.len())
    });
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! pair {
                        ($scenario:literal, $packed:ident, $vec:ident) => {
                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_editing/",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/ours_packed_string"
                                                                        )
                                                                    )]
                            fn $packed(bencher: Bencher) {
                                crate::$packed::<$bits>(bencher, $len);
                            }

                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_editing/",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/vec_u8"
                                                                        )
                                                                    )]
                            fn $vec(bencher: Bencher) {
                                crate::$vec::<$bits>(bencher, $len);
                            }
                        };
                    }

            pair!("set_position/aligned", set_middle_packed, set_middle_vec);
            pair!("push_boundary_probe", push_packed, push_vec);
            pair!("push_amortized", push_amortized_packed, push_amortized_vec);
            #[divan::bench(name = concat!("packed_editing/push_boundary_probe_nonzero/", stringify!($case), "/ours_packed_string"))]
            fn push_boundary_probe_nonzero(bencher: Bencher) {
                crate::push_packed_value::<$bits>(bencher, $len, crate::max_code($bits));
            }

            #[divan::bench(name = concat!("packed_editing/push_amortized_nonzero/", stringify!($case), "/ours_packed_string"))]
            fn push_amortized_nonzero(bencher: Bencher) {
                crate::push_amortized_packed_value::<$bits>(bencher, $len, crate::max_code($bits));
            }
            pair!("pop", pop_packed, pop_vec);
            pair!("insert_position/unaligned", insert_middle_packed, insert_middle_vec);
            pair!("remove_position/word_boundary_or_fallback", remove_middle_packed, remove_middle_vec);
            pair!("extend_bulk_25pct", extend_packed, extend_vec);
            #[divan::bench(name = concat!("packed_editing/extend_bulk_25pct_unaligned/", stringify!($case), "/ours_packed_string"))]
            fn extend_bulk_25pct_unaligned(bencher: Bencher) {
                crate::extend_packed_unaligned::<$bits>(bencher, $len);
            }
            #[divan::bench(name = concat!("packed_editing/extend_from_chars_concat/", stringify!($case), "/ours_packed_string"))]
            fn extend_from_chars_concat(bencher: Bencher) {
                crate::extend_from_chars_concat::<$bits>(bencher, $len);
            }
            #[divan::bench(name = concat!("packed_editing/extend_from_chars_concat_unaligned/", stringify!($case), "/ours_packed_string"))]
            fn extend_from_chars_concat_unaligned(bencher: Bencher) {
                crate::extend_from_chars_concat_unaligned::<$bits>(bencher, $len);
            }
        }
    };
}

crate::for_each_packed_bench_case!(define_case);

macro_rules! define_extend_length_case {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(args = [1, 16, 63, 64, 262144])]
            fn streaming_batches(bencher: Bencher, extension_len: usize) {
                crate::extend_with_length::<$bits>(bencher, extension_len, false);
            }

            #[divan::bench(args = [1, 16, 63, 64, 262144])]
            fn suffix_concat(bencher: Bencher, extension_len: usize) {
                crate::extend_with_length::<$bits>(bencher, extension_len, true);
            }

            #[divan::bench(args = [1, 16, 63, 64, 262144])]
            fn per_character_preallocated(bencher: Bencher, extension_len: usize) {
                crate::extend_per_character_with_length::<$bits>(bencher, extension_len);
            }
        }
    };
}

define_extend_length_case!(extend_length_bits_1, 1);
define_extend_length_case!(extend_length_bits_2, 2);
define_extend_length_case!(extend_length_bits_3, 3);
define_extend_length_case!(extend_length_bits_4, 4);
define_extend_length_case!(extend_length_bits_5, 5);
define_extend_length_case!(extend_length_bits_6, 6);
define_extend_length_case!(extend_length_bits_7, 7);
define_extend_length_case!(extend_length_bits_8, 8);

// Adjacent lengths exercise backing-word growth/shrink and partial tails.
macro_rules! define_edit_boundary_case {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(args = [63, 64, 65, 127, 128, 129])]
            fn insert_middle(bencher: Bencher, len: usize) {
                crate::insert_middle_packed::<$bits>(bencher, len);
            }

            #[divan::bench(args = [63, 64, 65, 127, 128, 129])]
            fn remove_middle(bencher: Bencher, len: usize) {
                crate::remove_middle_packed::<$bits>(bencher, len);
            }
        }
    };
}

define_edit_boundary_case!(edit_boundary_bits_1, 1);
define_edit_boundary_case!(edit_boundary_bits_2, 2);
define_edit_boundary_case!(edit_boundary_bits_3, 3);
define_edit_boundary_case!(edit_boundary_bits_4, 4);
define_edit_boundary_case!(edit_boundary_bits_5, 5);
define_edit_boundary_case!(edit_boundary_bits_6, 6);
define_edit_boundary_case!(edit_boundary_bits_7, 7);
define_edit_boundary_case!(edit_boundary_bits_8, 8);

macro_rules! define_push_shape_case {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("packed_editing/push_existing_word_zero/", stringify!($case), "/ours_packed_string"))]
            fn existing_word_zero(bencher: Bencher) {
                crate::push_existing_word::<$bits>(bencher, 0);
            }

            #[divan::bench(name = concat!("packed_editing/push_existing_word_nonzero/", stringify!($case), "/ours_packed_string"))]
            fn existing_word_nonzero(bencher: Bencher) {
                crate::push_existing_word::<$bits>(bencher, crate::max_code($bits));
            }

            #[divan::bench(name = concat!("packed_editing/push_word_growth_zero/", stringify!($case), "/ours_packed_string"))]
            fn word_growth_zero(bencher: Bencher) {
                crate::push_word_growth::<$bits>(bencher, 0);
            }

            #[divan::bench(name = concat!("packed_editing/push_word_growth_nonzero/", stringify!($case), "/ours_packed_string"))]
            fn word_growth_nonzero(bencher: Bencher) {
                crate::push_word_growth::<$bits>(bencher, crate::max_code($bits));
            }
        }
    };
}

macro_rules! define_cross_word_case {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("packed_editing/push_cross_word_zero/", stringify!($case), "/ours_packed_string"))]
            fn cross_word_zero(bencher: Bencher) {
                crate::push_cross_word::<$bits>(bencher, 0);
            }

            #[divan::bench(name = concat!("packed_editing/push_cross_word_nonzero/", stringify!($case), "/ours_packed_string"))]
            fn cross_word_nonzero(bencher: Bencher) {
                crate::push_cross_word::<$bits>(bencher, crate::max_code($bits));
            }
        }
    };
}

define_push_shape_case!(push_shape_bits_1, 1);
define_push_shape_case!(push_shape_bits_2, 2);
define_push_shape_case!(push_shape_bits_3, 3);
define_push_shape_case!(push_shape_bits_4, 4);
define_push_shape_case!(push_shape_bits_5, 5);
define_push_shape_case!(push_shape_bits_6, 6);
define_push_shape_case!(push_shape_bits_7, 7);
define_push_shape_case!(push_shape_bits_8, 8);

mod push_vec_bits_8 {
    use super::*;

    #[divan::bench(name = "packed_editing/push_existing_word_zero/push_shape_bits_8/vec_u8")]
    fn existing_word_zero(bencher: Bencher) {
        crate::push_existing_word_vec(bencher, 0);
    }

    #[divan::bench(name = "packed_editing/push_existing_word_nonzero/push_shape_bits_8/vec_u8")]
    fn existing_word_nonzero(bencher: Bencher) {
        crate::push_existing_word_vec(bencher, u8::MAX);
    }

    #[divan::bench(name = "packed_editing/push_word_growth_zero/push_shape_bits_8/vec_u8")]
    fn word_growth_zero(bencher: Bencher) {
        crate::push_word_growth_vec(bencher, 0);
    }

    #[divan::bench(name = "packed_editing/push_word_growth_nonzero/push_shape_bits_8/vec_u8")]
    fn word_growth_nonzero(bencher: Bencher) {
        crate::push_word_growth_vec(bencher, u8::MAX);
    }
}

define_cross_word_case!(cross_word_bits_3, 3);
define_cross_word_case!(cross_word_bits_5, 5);
define_cross_word_case!(cross_word_bits_6, 6);
define_cross_word_case!(cross_word_bits_7, 7);
