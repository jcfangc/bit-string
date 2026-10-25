pub(super) fn matching_lanes(
    windows: &[u64; 4],
    needle_first_word: u64,
    needle_first_word_mask: u64,
    candidate_count: usize,
) -> u8 {
    let mut matching_lanes = 0;
    for (lane, window) in windows.iter().enumerate().take(candidate_count) {
        if window & needle_first_word_mask == needle_first_word {
            matching_lanes |= 1 << lane;
        }
    }
    matching_lanes
}
