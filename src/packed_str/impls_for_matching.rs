//! Matching operations on packed, character-aligned views.
//!
//! Search strategies A–E were compared with paired seeds and equally weighted
//! workload classes, including mirrored early/late mismatches, for 64K and 1M
//! character haystacks. The selected strategy depends on operation, bit width,
//! and compile-time target features; each public search method documents its
//! measured dispatch. These are workload-specific results, not universal
//! rankings. Re-benchmark if workloads, compiler, or target features change.
//! `contains` remains on its existing any-candidate path.

use super::*;
use crate::traits::WordsFind;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use int_intervals::UsizeCO;

// Keep unmeasured mid-length needles on direct scanning; the measured long profile starts at 24.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const DUAL_ANCHOR_MIN_NEEDLE_LEN: usize = 24;

#[derive(Clone, Copy)]
enum SearchDirection {
    First,
    Last,
    Any,
}

fn search_aligned<const BITS: u8>(
    haystack: BitStr<'_>,
    needle: BitStr<'_>,
    direction: SearchDirection,
) -> Option<usize> {
    let width = usize::from(BITS);
    debug_assert_eq!(haystack.start() % width, 0);
    debug_assert_eq!(haystack.bit_len() % width, 0);
    debug_assert_eq!(needle.bit_len() % width, 0);

    if needle.bit_len() == 0 {
        return match direction {
            SearchDirection::First | SearchDirection::Any => Some(0),
            SearchDirection::Last => Some(haystack.bit_len() / width),
        };
    }
    if needle.bit_len() > haystack.bit_len() {
        return None;
    }

    let bit_offset = haystack.start() % crate::WORD_BITS;
    let first_word = haystack.start() / crate::WORD_BITS;
    let words = &haystack.source().words()[first_word..];
    let scan_bit_len = bit_offset + haystack.bit_len();
    let needle_first_word = [needle.get_chunk(0)];
    let mut verify = |candidate_bit_index: usize| {
        let Some(view_bit_index) = candidate_bit_index.checked_sub(bit_offset) else {
            return false;
        };
        view_bit_index.is_multiple_of(width) && haystack.matches_at_str(view_bit_index, needle)
    };

    let candidate = match direction {
        SearchDirection::First => words.find_first_word(
            scan_bit_len,
            &needle_first_word,
            needle.bit_len(),
            &mut verify,
        ),
        SearchDirection::Last => words.find_last_word(
            scan_bit_len,
            &needle_first_word,
            needle.bit_len(),
            &mut verify,
        ),
        SearchDirection::Any => words.find_any_candidate(
            scan_bit_len,
            &needle_first_word,
            needle.bit_len(),
            &mut verify,
        ),
    }?;

    Some((candidate - bit_offset) / width)
}

impl<C, const BITS: u8> PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn matches_at(&self, index: usize, pattern: Self) -> bool {
        let Some(bit_index) = index.checked_mul(usize::from(BITS)) else {
            return false;
        };
        if index > self.char_len() {
            return false;
        }
        self.bits.matches_at_str(bit_index, pattern.bits)
    }

    pub fn starts_with(&self, prefix: Self) -> bool {
        self.bits.starts_with_str(prefix.bits)
    }

    pub fn ends_with(&self, suffix: Self) -> bool {
        self.bits.ends_with_str(suffix.bits)
    }

    pub fn contains(&self, needle: Self) -> bool {
        search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::Any).is_some()
    }

    pub fn find(&self, needle: Self) -> Option<usize> {
        search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::First)
    }

    /// Finds the last character-aligned occurrence of `needle`.
    ///
    /// Paired mirrored A–E measurements select WordsFind (A) for BITS=1–6 on
    /// AVX2 targets and BITS=1–3 on baseline targets. AVX2 BITS=7 uses direct
    /// matching (D), while BITS=8 uses dual anchors (E) for needles of at least
    /// 24 characters; shorter needles use direct matching. Baseline BITS=4–8
    /// use direct matching (D).
    pub fn rfind(&self, needle: Self) -> Option<usize> {
        // The mirrored, paired A–E matrix selected WordsFind through BITS=6
        // on AVX2 targets and through BITS=3 on baseline targets. AVX2 uses
        // direct reverse scanning for BITS=7 and dual anchors for BITS=8.
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            if BITS <= 6 {
                return search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::Last);
            }
            if BITS == 8 {
                return self.rfind_dual_anchor(needle);
            }
        }

        #[cfg(not(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        )))]
        if BITS <= 3 {
            return search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::Last);
        }

        self.rfind_direct(needle)
    }

    fn rfind_direct(&self, needle: Self) -> Option<usize> {
        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len > haystack_len {
            return None;
        }

        (0..=haystack_len - needle_len)
            .rev()
            .find(|&index| self.matches_at(index, needle))
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn rfind_dual_anchor(&self, needle: Self) -> Option<usize> {
        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len > haystack_len {
            return None;
        }

        if needle_len < DUAL_ANCHOR_MIN_NEEDLE_LEN {
            return self.rfind_direct(needle);
        }

        let anchor_len = needle_len.min(64 / usize::from(BITS));

        let suffix_start = needle_len - anchor_len;
        let prefix = needle.slice(UsizeCO::checked_from_start_len(0, anchor_len).unwrap());
        let suffix =
            needle.slice(UsizeCO::checked_from_start_len(suffix_start, anchor_len).unwrap());
        (0..=haystack_len - needle_len).rev().find(|&index| {
            self.matches_at(index, prefix)
                && self.matches_at(index + suffix_start, suffix)
                && self.matches_at(index, needle)
        })
    }

    /// Runs the pre-dispatch word-search implementation for strategy benchmarks.
    #[cfg(feature = "benchmark-internals")]
    #[doc(hidden)]
    pub fn rfind_words_find_for_benchmark(&self, needle: Self) -> Option<usize> {
        search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::Last)
    }

    pub fn strip_prefix(&self, prefix: Self) -> Option<Self> {
        self.bits
            .strip_prefix_str(prefix.bits)
            .map(Self::from_valid_bit_str)
    }

    pub fn strip_suffix(&self, suffix: Self) -> Option<Self> {
        self.bits
            .strip_suffix_str(suffix.bits)
            .map(Self::from_valid_bit_str)
    }
}

#[cfg(test)]
mod tests_for_matching;
