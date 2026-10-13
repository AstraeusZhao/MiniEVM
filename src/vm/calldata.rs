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
