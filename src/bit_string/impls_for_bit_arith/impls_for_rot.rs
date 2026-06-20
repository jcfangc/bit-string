//! # Attempted `rotl` / `rotr` — why it didn't ship
//!
//! Circular bit rotation (`rotl` / `rotr`) was explored and ultimately
//! abandoned after hitting a fundamental tension between word-level
//! operations and partial-word bit strings.
//!
//! ## The problem
//!
//! `rotl(x, amount)` at the bit level is `dst[i] = src[(i + amount) % len]`.
//! Split `amount` into `word_shift` and `bit_shift`:
//!
//! ```text
//! amount = word_shift * WORD_BITS + bit_shift
//! ```
//!
//! Three approaches were tried:
//!
//! ### 1. `copy_bits` composition (correct, 1 alloc)
//!
//! ```rust
//! copy_bits(src, amount,  dst, 0,            len - amount); // suffix → front
//! copy_bits(src, 0,       dst, len - amount, amount);       // prefix → end
//! ```
//!
//! Correct for all bit lengths.  The only downside is one temporary
//! allocation for the output buffer (and a second one for `assign`'s
//! scratch copy).  This is what the committed (and later reverted)
//! implementation used.
//!
//! ### 2. Word-rotate + carry-chain (0 alloc, buggy)
//!
//! ```text
//! 1. bits.rotate_left(word_shift)   // word-level rotation
//! 2. right-shift carry chain        // bit-level adjustment
//! 3. mask_unused_bits
//! ```
//!
//! **Failure mode:** the word rotation moves *full* u64 words, but the
//! last word of a non-64-aligned bit string has unused high bits.  After
//! rotation those padding bits land in valid positions and get mixed in
//! by the carry chain.  Example with a 65-bit all-ones string rotated
//! by 64:
//!
//! ```text
//! before: [0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF]
//! mask:   [0xFFFF_FFFF_FFFF_FFFF, 0x0000_0000_0000_0001]
//! rotate_left(1): [0x0000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFF]
//! mask:   [0x0000_0000_0000_0001, 0x0000_0000_0000_0001]
//! // The 63 valid bits that were in word 0 are gone — they moved into
//! // word 1 where mask_unused_bits zeroed them.
//! ```
//!
//! ### 3. `ptr::copy` word-level memmove (0 alloc, buggy)
//!
//! Save the prefix words to a stack/heap buffer, memmove the suffix
//! left, memmove the saved prefix to the tail, then apply a carry
//! chain for `bit_shift`.
//!
//! **Failure mode:** when `bit_shift > 0` the word at index `word_shift`
//! is *shared* between prefix (high bits) and suffix (low bits).  We
//! need to save `word_shift + 1` words to capture the full prefix, but
//! writing them back at the tail overflows the buffer by one word:
//!
//! ```text
//! save_words = word_shift + 1
//! dst_start  = word_cnt - word_shift
//! dst_end    = dst_start + save_words = word_cnt + 1   // overflow!
//! ```
//!
//! Even with correct bounds, the shared boundary word cannot be split
//! by word-level operations — the prefix and suffix are interleaved at
//! the bit level inside that single u64.
//!
//! ## Conclusion
//!
//! Bit-level rotation on a non-word-aligned bit string cannot be done
//! in-place with word-granularity operations without a temporary
//! allocation.  The only correct 0-alloc algorithm would be a
//! cycle-following (juggling) rotation at the *bit* level — O(n) bit
//! operations, far slower than one allocation.
//!
//! The `copy_bits` approach (approach 1) is the right engineering
//! trade-off: one allocation, trivially correct, and fast enough for
//! all practical purposes.  If a future optimiser eliminates the
//! allocation through inlining / escape analysis, even better.
//!
//! ## What a future implementation should look like
//!
//! ```rust
//! // impls_for_rotl.rs (public API)
//! // impls_for_rotl/funcs_for_rotl_core.rs (owned / assign)
//! //
//! // owned:  alloc zeroed buffer, 2× copy_bits, mask
//! // assign: call owned, copy_from_slice back
//! // rotr:   delegating to rotl with reversed amount
//! ```
