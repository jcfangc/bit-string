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

// ---------------------------------------------------------------------------
// BitString ↔ BitStr hash consistency (BitString delegates to as_bit_str)
// ---------------------------------------------------------------------------

#[test]
fn bit_string_hash_equals_bit_str_hash() {
    let bs = BitString::try_from("101100101").unwrap();
    assert_eq!(hash_one(&bs), hash_one(&bs.as_bit_str()));
}

#[test]
fn large_bit_string_matches_bit_str() {
    let mut bs = BitString::zeros(1024);
    for i in (0..1024).step_by(128) {
        bs.set(i, true);
    }
    assert_eq!(hash_one(&bs), hash_one(&bs.as_bit_str()));
}

#[test]
fn slice_to_bit_string_roundtrip_preserves_hash() {
    let source = BitString::try_from("110010101111").unwrap();
    let v = source.as_bit_str().slice_from(2).slice_until(8);
    let owned = v.to_bit_string();
    assert_eq!(hash_one(&v), hash_one(&owned));
}

// ---------------------------------------------------------------------------
// Unaligned views
// ---------------------------------------------------------------------------

/// Unaligned views must hash identically to aligned views with the same content.
#[test]
fn unaligned_view_hashes_same_as_aligned_with_same_content() {
    let mut source = BitString::zeros(200);
    // Set bits that span the tail region — the buggy old code computed the
    // wrong tail_start (off by s bits) and wrong tail_bits (bit_len%WORD_BITS
    // instead of remaining%WORD_BITS) for unaligned views.
    source.set(192, true); // lives in the tail of a bit_len=194, s=3 view
    source.set(193, true);

    // Unaligned: start=3, bit_len=194  (first: 61, mid: 2×64=128, tail: 5)
    let unaligned = source.as_bit_str().slice_from(3).slice_until(197);

    // Aligned equivalent via roundtrip.
    let owned = unaligned.to_bit_string();
    let aligned = owned.as_bit_str();

    assert_eq!(unaligned, aligned);
    assert_eq!(hash_one(&unaligned), hash_one(&aligned));
}
