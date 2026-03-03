//! Bytecode structural analysis used by control-flow validation.

/// Immutable bytecode plus a bitmap of executable `JUMPDEST` positions.
#[derive(Debug, Clone)]
pub(super) struct AnalyzedCode {
    bytes: Vec<u8>,
    jumpdest: Vec<bool>,
}

impl AnalyzedCode {
    pub(super) fn new(bytes: Vec<u8>) -> Self {
        let mut jumpdest = vec![false; bytes.len()];
        let mut cursor = 0;
        while cursor < bytes.len() {
            let opcode = bytes[cursor];
            if opcode == 0x5b {
                jumpdest[cursor] = true;
            }
            cursor += 1;
            if (0x60..=0x7f).contains(&opcode) {
                cursor = cursor.saturating_add((opcode - 0x5f) as usize);
            }
        }
        Self { bytes, jumpdest }
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(super) fn is_valid_jumpdest(&self, offset: usize) -> bool {
        self.jumpdest.get(offset).copied().unwrap_or(false)
    }
}
