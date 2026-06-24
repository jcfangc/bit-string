use core::hash::{Hash, Hasher};

use crate::BitString;

/// A trivial deterministic hasher for testing hash consistency.
struct TestHasher(u64);

impl Hasher for TestHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
        }
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = self.0.wrapping_mul(31).wrapping_add(i);
    }

    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }
}

fn hash_one(v: &impl Hash) -> u64 {
    let mut h = TestHasher(0);
    v.hash(&mut h);
    h.finish()
}

#[test]
fn equal_views_have_same_hash() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101001").unwrap();
    assert_eq!(hash_one(&a.as_bit_str()), hash_one(&b.as_bit_str()));
}

#[test]
fn different_content_has_different_hash() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("111001").unwrap();
    assert_ne!(hash_one(&a.as_bit_str()), hash_one(&b.as_bit_str()));
}

#[test]
fn different_lengths_have_different_hash() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101").unwrap();
    assert_ne!(hash_one(&a.as_bit_str()), hash_one(&b.as_bit_str()));
}

#[test]
fn empty_views_from_different_sources_have_same_hash() {
    let a = BitString::new();
    let b = BitString::try_from("101").unwrap();
    let empty = b.as_bit_str().slice_from(0).slice_until(0);
    assert_eq!(hash_one(&a.as_bit_str()), hash_one(&empty));
}

#[test]
fn offset_views_with_same_content_have_same_hash() {
    let bits = BitString::try_from("110010").unwrap();
    let v1 = bits.as_bit_str().slice_from(1).slice_until(4);
    let v2 = bits.as_bit_str().slice_from(1).slice_until(4);
    assert_eq!(hash_one(&v1), hash_one(&v2));
}

#[test]
fn views_across_word_boundaries() {
    let mut a = BitString::zeros(130);
    a.set(62, true);
    a.set(63, true);
    a.set(64, true);

    let mut b = BitString::zeros(130);
    b.set(62, true);
    b.set(63, true);
    b.set(64, true);

    assert_eq!(hash_one(&a.as_bit_str()), hash_one(&b.as_bit_str()));
}
