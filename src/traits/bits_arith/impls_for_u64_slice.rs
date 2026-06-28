use alloc::vec::Vec;

use super::BitsArith;
use super::funcs_for_binary_core::{OP_AND, OP_OR, OP_XOR, assign, owned};
use super::funcs_for_count_ones;
use super::funcs_for_not_core;
use super::funcs_for_shl_core;
use super::funcs_for_shr_core;
use super::funcs_for_value_words_core::{
    leading_value_words as leading_value_words_core,
    trailing_value_words as trailing_value_words_core,
};

impl BitsArith for [u64] {
    #[inline]
    fn and(&self, rhs: &[u64]) -> Vec<u64> {
        owned::<OP_AND>(self, rhs)
    }

    #[inline]
    fn and_assign(&mut self, rhs: &[u64]) {
        assign::<OP_AND>(self, rhs);
    }

    #[inline]
    fn or(&self, rhs: &[u64]) -> Vec<u64> {
        owned::<OP_OR>(self, rhs)
    }

    #[inline]
    fn or_assign(&mut self, rhs: &[u64]) {
        assign::<OP_OR>(self, rhs);
    }

    #[inline]
    fn xor(&self, rhs: &[u64]) -> Vec<u64> {
        owned::<OP_XOR>(self, rhs)
    }

    #[inline]
    fn xor_assign(&mut self, rhs: &[u64]) {
        assign::<OP_XOR>(self, rhs);
    }

    #[inline]
    fn not(&self, bit_len: usize) -> Vec<u64> {
        funcs_for_not_core::owned(self, bit_len)
    }

    #[inline]
    fn not_assign(&mut self, bit_len: usize) {
        funcs_for_not_core::assign(self, bit_len);
    }

    #[inline]
    fn shl(&self, bit_len: usize, amount: usize) -> Vec<u64> {
        funcs_for_shl_core::owned(self, bit_len, amount)
    }

    #[inline]
    fn shl_assign(&mut self, bit_len: usize, amount: usize) {
        funcs_for_shl_core::assign(self, bit_len, amount);
    }

    #[inline]
    fn shr(&self, bit_len: usize, amount: usize) -> Vec<u64> {
        funcs_for_shr_core::owned(self, bit_len, amount)
    }

    #[inline]
    fn shr_assign(&mut self, bit_len: usize, amount: usize) {
        funcs_for_shr_core::assign(self, bit_len, amount);
    }

    #[inline]
    fn count_ones(&self, bit_len: usize) -> usize {
        funcs_for_count_ones::count_ones(self, bit_len)
    }

    #[inline]
    fn leading_value_words<const FILL: u64>(&self) -> usize {
        leading_value_words_core::<FILL>(self)
    }

    #[inline]
    fn trailing_value_words<const FILL: u64>(&self) -> usize {
        trailing_value_words_core::<FILL>(self)
    }
}
