use crate::{BitStr, WORD_BITS, low_mask, traits::WordsEdit};

mod funcs_for_candidate_core;

struct NeedlePrefix<'a> {
    bits: BitStr<'a>,
    first_word: u64,
    first_word_mask: u64,
}

impl<'a> NeedlePrefix<'a> {
    fn new<const NEEDLE_START_ZERO: bool>(bits: BitStr<'a>) -> Self {
        let first_word_bit_len = bits.bit_len().min(WORD_BITS);
        let first_word_mask = low_mask(first_word_bit_len);
        let first_word = if NEEDLE_START_ZERO {
            bits.source().words().read_word_at::<false>(0)
        } else {
            bits.get_chunk(0)
        };
        Self {
            bits,
            first_word: first_word & first_word_mask,
            first_word_mask,
        }
    }

    fn matches_at<const HAYSTACK_START_ZERO: bool>(
        &self,
        haystack: BitStr<'_>,
        bit_start: usize,
    ) -> bool {
        let candidate_word = if HAYSTACK_START_ZERO {
            haystack.source().words().read_word_at::<false>(bit_start)
        } else {
            haystack.get_chunk(bit_start)
        };
        let candidate_prefix = candidate_word & self.first_word_mask;
        candidate_prefix == self.first_word
            && (self.bits.bit_len() <= WORD_BITS || haystack.matches_at_str(bit_start, self.bits))
    }
}

fn last_candidate<const BITS: u8>(haystack: BitStr<'_>, needle: BitStr<'_>) -> Option<usize> {
    let width = usize::from(BITS);
    let haystack_char_len = haystack.bit_len() / width;
    let needle_char_len = needle.bit_len() / width;
    haystack_char_len.checked_sub(needle_char_len)
}

fn candidate_windows<const BITS: u8, const HAYSTACK_START_ZERO: bool>(
    haystack: BitStr<'_>,
    first_char: usize,
    candidate_count: usize,
) -> [u64; 4] {
    let width = usize::from(BITS);
    core::array::from_fn(|lane| {
        if lane < candidate_count {
            let bit_start = (first_char + lane) * width;
            if HAYSTACK_START_ZERO {
                haystack.source().words().read_word_at::<false>(bit_start)
            } else {
                haystack.get_chunk(bit_start)
            }
        } else {
            0
        }
    })
}

pub(super) fn find<
    const BITS: u8,
    const HAYSTACK_START_ZERO: bool,
    const NEEDLE_START_ZERO: bool,
>(
    haystack: BitStr<'_>,
    needle: BitStr<'_>,
) -> Option<usize> {
    if needle.bit_len() == 0 {
        return Some(0);
    }

    let width = usize::from(BITS);
    let last_start = last_candidate::<BITS>(haystack, needle)?;
    if haystack.matches_at_str(0, needle) {
        return Some(0);
    }
    let needle = NeedlePrefix::new::<NEEDLE_START_ZERO>(needle);
    let mut first_char = 1;

    while first_char <= last_start {
        let candidate_count = (last_start - first_char).min(3) + 1;
        let windows =
            candidate_windows::<BITS, HAYSTACK_START_ZERO>(haystack, first_char, candidate_count);
        let matching_lanes = funcs_for_candidate_core::matching_lanes(
            &windows,
            needle.first_word,
            needle.first_word_mask,
            candidate_count,
        );

        for lane in 0..candidate_count {
            if matching_lanes & (1 << lane) != 0 {
                let char_start = first_char + lane;
                if needle.matches_at::<HAYSTACK_START_ZERO>(haystack, char_start * width) {
                    return Some(char_start);
                }
            }
        }

        first_char += candidate_count;
    }
    None
}

pub(super) fn rfind<
    const BITS: u8,
    const HAYSTACK_START_ZERO: bool,
    const NEEDLE_START_ZERO: bool,
>(
    haystack: BitStr<'_>,
    needle: BitStr<'_>,
) -> Option<usize> {
    if needle.bit_len() == 0 {
        return Some(haystack.bit_len() / usize::from(BITS));
    }

    let width = usize::from(BITS);
    let last_start = last_candidate::<BITS>(haystack, needle)?;
    if haystack.matches_at_str(last_start * width, needle) {
        return Some(last_start);
    }
    if last_start == 0 {
        return None;
    }
    let needle = NeedlePrefix::new::<NEEDLE_START_ZERO>(needle);
    let mut batch_end = last_start - 1;

    loop {
        let batch_start = batch_end.saturating_sub(3);
        let candidate_count = batch_end - batch_start + 1;
        let windows =
            candidate_windows::<BITS, HAYSTACK_START_ZERO>(haystack, batch_start, candidate_count);
        let matching_lanes = funcs_for_candidate_core::matching_lanes(
            &windows,
            needle.first_word,
            needle.first_word_mask,
            candidate_count,
        );

        for lane in (0..candidate_count).rev() {
            if matching_lanes & (1 << lane) != 0 {
                let char_start = batch_start + lane;
                if needle.matches_at::<HAYSTACK_START_ZERO>(haystack, char_start * width) {
                    return Some(char_start);
                }
            }
        }

        if batch_start == 0 {
            break;
        }
        batch_end = batch_start - 1;
    }
    None
}

pub(super) fn contains<
    const BITS: u8,
    const HAYSTACK_START_ZERO: bool,
    const NEEDLE_START_ZERO: bool,
>(
    haystack: BitStr<'_>,
    needle: BitStr<'_>,
) -> bool {
    if needle.bit_len() == 0 {
        return true;
    }

    find::<BITS, HAYSTACK_START_ZERO, NEEDLE_START_ZERO>(haystack, needle).is_some()
}

#[cfg(test)]
mod tests_for_inner;
