use crate::primitives::{Address, Word};

/// Event data emitted by a successful execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub address: Address,
    pub topics: Vec<Word>,
    pub data: Vec<u8>,
}
