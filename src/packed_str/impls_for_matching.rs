//! Matching operations on packed, character-aligned views.
//!
//! The default `find_with` and `rfind_with` dispatch preserves the measured
//! packed-search policy: on AVX2 targets, `find_with` uses `WordsFind` for
//! BITS=1..=7 and `rfind_with` for BITS=1..=6; baseline targets use it through
//! BITS=6 and BITS=3, respectively. AVX2 BITS=8 uses two-end anchors for
//! needles of at least 24 characters, while shorter needles and other widths
//! outside those ranges use direct character-aligned matching. These choices
//! reflect the benchmarked workloads and target configuration, not universal
//! rankings. Re-measure if either changes.
//!
//! `FindOptions` hints currently do not alter this default dispatch. For
//! `contains_with`, `LocationHint` can choose which end is scanned first, and
//! `AnchorHint::Suffix` checks a short suffix before full verification. Prefix
//! hints use the existing forward verification order.

use super::*;
use crate::traits::WordsFind;
use int_intervals::UsizeCO;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
const DUAL_ANCHOR_MIN_NEEDLE_LEN: usize = 24;

#[derive(Clone, Copy)]
enum SearchDirection {
    First,
    Last,
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
            SearchDirection::First => Some(0),
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
    }?;

    Some((candidate - bit_offset) / width)
}
use crate::{ContainsOptions, FindOptions};

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

    /// Returns whether `needle` occurs in this view using the supplied hints.
    pub fn contains_with(&self, needle: Self, options: ContainsOptions) -> bool {
        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len == 0 {
            return true;
        }
        if needle_len > haystack_len {
            return false;
        }

        let last_start = haystack_len - needle_len;
        if options.anchor() != crate::AnchorHint::Suffix
            && options.location() != crate::LocationHint::NearEnd
        {
            return (0..=last_start).any(|index| self.matches_at(index, needle));
        }

        let suffix_anchor = if options.anchor() == crate::AnchorHint::Suffix {
            let anchor_len = needle_len.min(crate::WORD_BITS / usize::from(BITS));
            let anchor_start = needle_len - anchor_len;
            let interval = UsizeCO::checked_from_start_len(anchor_start, anchor_len).unwrap();
            Some((anchor_start, needle.slice(interval)))
        } else {
            None
        };

        let matches_at = |index| {
            if let Some((anchor_start, anchor)) = suffix_anchor {
                if !self.matches_at(index + anchor_start, anchor) {
                    return false;
                }
                if anchor_start == 0 {
                    return true;
                }
            }
            self.matches_at(index, needle)
        };

        match options.location() {
            crate::LocationHint::NearEnd => (0..=last_start).rev().any(matches_at),
            crate::LocationHint::Unknown | crate::LocationHint::NearStart => {
                (0..=last_start).any(matches_at)
            }
        }
    }

    /// Returns the first matching character index using the supplied hints.
    pub fn find_with(&self, needle: Self, _options: FindOptions) -> Option<usize> {
        // Keep the previously benchmarked default dispatch. Workload hints are
        // advisory and are not used by the first version of this planner.
        if BITS <= 6 {
            return search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::First);
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            if BITS == 7 {
                return search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::First);
            }
            if BITS == 8 {
                return self.find_dual_anchor(needle);
            }
        }

        self.find_direct(needle)
    }

    fn find_direct(&self, needle: Self) -> Option<usize> {
        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len > haystack_len {
            return None;
        }

        (0..=haystack_len - needle_len).find(|&index| self.matches_at(index, needle))
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn find_dual_anchor(&self, needle: Self) -> Option<usize> {
        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len > haystack_len {
            return None;
        }

        if needle_len < DUAL_ANCHOR_MIN_NEEDLE_LEN {
            return self.find_direct(needle);
        }

        let anchor_len = needle_len.min(64 / usize::from(BITS));
        let suffix_start = needle_len - anchor_len;
        let prefix = needle.slice(UsizeCO::checked_from_start_len(0, anchor_len).unwrap());
        let suffix =
            needle.slice(UsizeCO::checked_from_start_len(suffix_start, anchor_len).unwrap());
        (0..=haystack_len - needle_len).find(|&index| {
            self.matches_at(index, prefix)
                && self.matches_at(index + suffix_start, suffix)
                && self.matches_at(index, needle)
        })
    }

    /// Returns the last matching character index using the supplied hints.
    pub fn rfind_with(&self, needle: Self, _options: FindOptions) -> Option<usize> {
        // As with `find_with`, retain the measured default dispatch while
        // leaving workload-hint planning for a separate performance patch.
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
