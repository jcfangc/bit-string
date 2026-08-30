#![no_std]

use bit_string::BitString;

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_from_bool_slice(values: *const bool, len: usize) -> BitString {
    let values = unsafe { core::slice::from_raw_parts(values, len) };
    BitString::from(values)
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_from_str(value: *const u8, len: usize) -> BitString {
    let bytes = unsafe { core::slice::from_raw_parts(value, len) };
    BitString::try_from(unsafe { core::str::from_utf8_unchecked(bytes) }).unwrap()
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_xor(lhs: *const BitString, rhs: *const BitString) -> BitString {
    unsafe { (&*lhs).xor(&*rhs).unwrap() }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_not(value: *const BitString) -> BitString {
    unsafe { (&*value).not() }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_shl(value: *const BitString, amount: usize) -> BitString {
    unsafe { (&*value).shl(amount) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_shr(value: *const BitString, amount: usize) -> BitString {
    unsafe { (&*value).shr(amount) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_slice_from(value: *const BitString, start: usize) -> BitString {
    unsafe { (&*value).slice_from(start) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_matches_at(
    lhs: *const BitString,
    index: usize,
    rhs: *const BitString,
) -> bool {
    unsafe { (&*lhs).matches_at_string(index, &*rhs) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_contains(lhs: *const BitString, rhs: *const BitString) -> bool {
    unsafe { (&*lhs).contains_string(&*rhs) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_find(
    lhs: *const BitString,
    rhs: *const BitString,
) -> Option<usize> {
    unsafe { (&*lhs).find_string(&*rhs) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_rfind(
    lhs: *const BitString,
    rhs: *const BitString,
) -> Option<usize> {
    unsafe { (&*lhs).rfind_string(&*rhs) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_cmp(
    lhs: *const BitString,
    rhs: *const BitString,
) -> core::cmp::Ordering {
    unsafe { (&*lhs).cmp_string(&*rhs) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_count_ones(value: *const BitString) -> usize {
    unsafe { (&*value).count_ones() }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_leading_zeros(value: *const BitString) -> usize {
    unsafe { (&*value).leading_zeros() }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe fn codegen_bit_string_trailing_zeros(value: *const BitString) -> usize {
    unsafe { (&*value).trailing_zeros() }
}
