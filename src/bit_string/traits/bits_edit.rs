/// Bit-level editing operations on `[u64]` backing storage.
///
/// All bit indices are logical positions in the flat bit string (not word
/// indices). Word boundaries are handled transparently — callers work with
/// logical bit positions and the trait translates them to word-level accesses.
pub(crate) trait BitsEdit {
    /// Zeros words beyond `word_len(len)` and masks the last used word.
    ///
    /// `len` is the **total** bit-string length. Words whose bits are entirely
    /// beyond `len` are zeroed in full. When the last used word is partially
    /// filled (`len % WORD_BITS != 0`) only the low remainder bits are
    /// preserved. When `len` is zero all words are cleared.
    fn mask_unused_bits(&mut self, len: usize);

    /// Reads a single bit at `index`, returning `true` for 1 and `false` for 0.
    fn read_bit_at(&self, index: usize) -> bool;

    /// Sets the bit at `index` to `value` (`true` = 1, `false` = 0).
    fn set_bit_at(&mut self, index: usize, value: bool);

    /// Reads up to `WORD_BITS` (64) bits starting at `bit_start`.
    ///
    /// When `bit_start` is word-aligned the result comes from a single word.
    /// Otherwise it spans two consecutive words, shifting and combining them.
    /// Bits past the end of `self` are silently treated as zero.
    fn read_word_at(&self, bit_start: usize) -> u64;

    /// Writes the low `len` bits of `value` into `self` starting at `bit_start`.
    ///
    /// Existing bits in the destination range are preserved (the write uses
    /// bitwise OR). `len` is clamped to `WORD_BITS` via
    /// [`BitsMask::low_mask`]; callers
    /// must ensure `len <= WORD_BITS`. When `bit_start` is not word-aligned
    /// the value is split across two consecutive words.
    fn write_word_at(&mut self, bit_start: usize, value: u64, len: usize);

    /// Captures a snapshot of `len` bits starting at `start` for deferred
    /// paste via [`BitsCopied::paste_to`].
    fn copy_bits(&self, start: usize, len: usize) -> BitsCopied<'_>;

    /// Clears `len` bits starting at `start`. `len` must be > 0.
    ///
    /// When the range fits within a single word only the affected bits are
    /// cleared. For multi-word ranges interior words are zeroed entirely and
    /// the two edge words are partially cleared.
    fn clear_bits_at(&mut self, start: usize, len: usize);

    /// Shifts bits in `[start, start + count)` one position to the right
    /// (toward higher indices), inserting space at `start`.
    ///
    /// The vacated bit at `start` is zero-filled. The bit that was at
    /// `start + count - 1` moves to `start + count`. `count` must be > 0.
    fn shift_right_in_place(&mut self, start: usize, count: usize);

    /// Shifts bits in `[start, start + count)` one position to the left
    /// (toward lower indices), closing the gap at `start`.
    ///
    /// The bit that was at `start` is dropped. The vacated bit at
    /// `start + count - 1` is zero-filled. `count` must be > 0.
    fn shift_left_in_place(&mut self, start: usize, count: usize);
}

pub(crate) mod copy;
pub(crate) mod funcs_for_alloc;
pub(crate) mod impls_for_bits_edit;

pub(crate) use copy::BitsCopied;
pub(crate) use funcs_for_alloc::*;
