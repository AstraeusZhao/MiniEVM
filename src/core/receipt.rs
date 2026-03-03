use crate::state::LogEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitReason {
    Succeed,
    Revert,
    Error,
}

/// Result of an entire transaction boundary, including committed logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub reason: ExitReason,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<LogEntry>,
}
