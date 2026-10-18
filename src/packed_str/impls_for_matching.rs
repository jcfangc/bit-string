use super::*;
use crate::traits::WordsFind;

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

    pub fn rfind(&self, needle: Self) -> Option<usize> {
        // The native (`-C target-cpu=native`) matrix used 64K/1M haystacks and
        // 16 paired input seeds. Its equal-weight workload score favored
        // WordsFind for BITS=1..=5. Direct character scanning reduced that
        // score by 7.9% for BITS=6 and about 33% for BITS=7/8 versus WordsFind.
        // For BITS=7/8 it was statistically tied with suffix-anchor scanning,
        // so use the simpler direct scan. Re-benchmark if target or workload
        // assumptions change.
        if BITS <= 5 {
            return search_aligned::<BITS>(self.bits, needle.bits, SearchDirection::Last);
        }

        let needle_len = needle.char_len();
        let haystack_len = self.char_len();
        if needle_len > haystack_len {
            return None;
        }

        (0..=haystack_len - needle_len)
            .rev()
            .find(|&index| self.matches_at(index, needle))
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
