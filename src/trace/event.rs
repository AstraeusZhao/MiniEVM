use crate::primitives::{Address, Word};

/// Pre-instruction state supplied to a tracer before base gas is charged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub address: Address,
    pub depth: u16,
    pub program_counter: usize,
    pub opcode: u8,
    pub gas_remaining: u64,
    pub stack: Vec<Word>,
}
