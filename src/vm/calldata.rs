use crate::primitives::Word;

/// Loads a zero-right-padded 32-byte word from calldata.
pub(super) fn load_word(input: &[u8], offset: usize) -> Word {
    let mut word = [0u8; 32];
    if let Some(slice) = input.get(offset..) {
        let length = slice.len().min(word.len());
        word[..length].copy_from_slice(&slice[..length]);
    }
    Word::from_be_slice(&word)
}

/// Copies a bounded source range and pads bytes beyond the source with zero.
pub(super) fn copy_padded(source: &[u8], offset: usize, size: usize) -> Vec<u8> {
    let mut output = vec![0; size];
    if let Some(available) = source.get(offset..) {
        let length = available.len().min(size);
        output[..length].copy_from_slice(&available[..length]);
    }
    output
}
