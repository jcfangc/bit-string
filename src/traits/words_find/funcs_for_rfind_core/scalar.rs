use crate::WORD_BITS;

pub(super) fn scalar_rfind<F>(
    haystack: &[u64],
    needle_key: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
    for i in (0..=start_word).rev() {
        let base = i * WORD_BITS;
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        // Note: the SIMD backends compute max_shift differently —
        // `WORD_BITS.min(last_start - base + 1)` — to process
        // shifts in SIMD-sized chunks (2 or 4), relying on
        // `pos <= last_start` to skip out-of-range positions.
        let max_shift = (last_start - base).min(WORD_BITS - 1);
        for shift in (0..=max_shift).rev() {
            let pos = base + shift;
            let window = if shift == 0 {
                w0
            } else {
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_key && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}
