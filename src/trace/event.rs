use crate::primitives::Word;

/// Instruction snapshot supplied to registered tracer implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub program_counter: usize,
    pub opcode: u8,
    pub gas_remaining: u64,
    pub stack: Vec<Word>,
}
