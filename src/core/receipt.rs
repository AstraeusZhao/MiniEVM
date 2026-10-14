use crate::{error::VmError, state::LogEntry};

/// Transaction completion category; inspect `ExecutionResult::error` for failures.
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
    /// Validation or exceptional execution failure. `None` for success and REVERT.
    pub error: Option<VmError>,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<LogEntry>,
}

impl ExecutionResult {
    pub(crate) fn failed(error: VmError, gas_used: u64) -> Self {
        Self {
            reason: ExitReason::Error,
            error: Some(error),
            gas_used,
            output: vec![],
            logs: vec![],
        }
    }
}
