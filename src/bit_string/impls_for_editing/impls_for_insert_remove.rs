use crate::funcs_for_bits::*;
use crate::traits::*;

use super::*;

impl BitString {
    /// Inserts the low `width` bits of `value`, shifting the suffix once.
    ///
    /// `width` must be at most one word. The index is clamped to the end;
    /// zero width is a no-op. Growth is checked before modifying the payload.
    pub(crate) fn insert_bits(&mut self, index: usize, value: u64, width: usize) {
        assert!(
            width <= WORD_BITS,
            "cannot insert more than one word of bits"
        );
        let index = index.min(self.bit_len);
        if index == self.bit_len {
            self.push_bits(value, width);
            return;
        }
        if width == 0 {
            return;
        }

        let new_len = self
            .bit_len
            .checked_add(width)
            .expect("bit string length overflow");
        // Allocate before changing existing bits, retaining spare capacity.
        self.words.resize(word_len(new_len), 0);

        let first_word = index / WORD_BITS;
        let prefix_mask = crate::low_mask(index % WORD_BITS);
        let prefix = self.words[first_word] & prefix_mask;
        // Clear the boundary prefix before shifting so it cannot enter the gap.
        // The word-level shift preserves overlapping input.
        self.words[first_word] &= !prefix_mask;
        self.words[first_word..].shl_assign(new_len - first_word * WORD_BITS, width);
        self.words[first_word] |= prefix;
        // The shift leaves [index, index + width) zero, as required by OR writes.
        self.words.write_word_at::<false>(index, value, width);
        self.bit_len = new_len;
    }

    /// Removes `width` bits at `index`, shifting the suffix once.
    ///
    /// `width` must be at most one word. Zero width is a no-op; otherwise
    /// the entire range must exist. Invalid arguments panic before mutation.
    pub(crate) fn remove_bits(&mut self, index: usize, width: usize) {
        assert!(
            width <= WORD_BITS,
            "cannot remove more than one word of bits"
        );
        if width == 0 {
            return;
        }
        assert!(
            index <= self.bit_len && width <= self.bit_len - index,
            "bit string removal range out of bounds"
        );

        let first_word = index / WORD_BITS;
        let prefix_mask = crate::low_mask(index % WORD_BITS);
        let prefix = self.words[first_word] & prefix_mask;
        // The word-level shift runs low to high, preserving overlapping input.
        // At every bit position >= index, it reads the old position + width.
        self.words[first_word..].shr_assign(self.bit_len - first_word * WORD_BITS, width);
        // Removed bits may have shifted into the prefix: replace, rather than OR.
        self.words[first_word] = (self.words[first_word] & !prefix_mask) | prefix;

        let new_len = self.bit_len - width;
        let new_words = word_len(new_len);
        if new_words < self.words.len() {
            self.truncate_words(new_words);
        }
        self.words.mask_unused_bits(new_len);
        self.bit_len = new_len;
    }

    /// Inserts a single bit at `index`, shifting the tail right.
    ///
    /// `index` is clamped to `[0, self.bit_len()]` — an out-of-bounds index
    /// pushes at the end.
    pub fn insert(&mut self, index: usize, value: bool) {
        let index = index.min(self.bit_len);

        if index == self.bit_len {
            self.push(value);
            return;
        }

        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // In-place fast path: word count unchanged (~98% of operations).
        if new_words == self.words.len() {
            // Ensure the last word is present (Vec::len == new_words already).
            self.words.resize(new_words, 0);
            self.words.shift_right_in_place(index, self.bit_len - index);
            self.words.set_bit_at(index, value);
            self.bit_len = new_len;
            return;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, index).paste_to(&mut bits, 0);
        bits.set_bit_at(index, value);
        self.words
            .copy_bits(index, self.bit_len - index)
            .paste_to(&mut bits, index + 1);

        self.words = bits;
        self.bit_len = new_len;
    }

    /// Removes the bit at `index`, shifting the tail left.
    ///
    /// Returns `false` (without modifying `self`) when `index >= self.bit_len()`.
    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.bit_len {
            return false;
        }

        let value = self.words.read_bit_at(index);
        let new_len = self.bit_len - 1;
        let new_words = word_len(new_len);

        // In-place fast path: word count unchanged.
        if new_words == self.words.len() {
            self.words
                .shift_left_in_place(index + 1, self.bit_len - index - 1);
            self.bit_len = new_len;
            self.words.mask_unused_bits(self.bit_len);
            return value;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, index).paste_to(&mut bits, 0);
        self.words
            .copy_bits(index + 1, self.bit_len - index - 1)
            .paste_to(&mut bits, index);

        self.words = bits;
        self.bit_len = new_len;

        value
    }
}

#[cfg(test)]
mod tests_for_insert;

#[cfg(test)]
mod tests_for_remove;

#[cfg(test)]
mod tests_for_edit_bits;
