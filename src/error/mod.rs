use crate::primitives::Address;
use std::fmt;

/// Checked transaction and execution failures in MiniEVM's simplified model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    OutOfGas,
    StackUnderflow,
    StackOverflow,
    InvalidOpcode(u8),
    InvalidJump(usize),
    CodeOutOfBounds,
    MemoryLimitExceeded,
    ReturnDataOutOfBounds,
    InsufficientBalance { account: Address, needed: u64, available: u64 },
    InvalidNonce { account: Address, expected: u64, actual: u64 },
    NonceOverflow,
    ValueOverflow,
    CallDepthExceeded,
    RevertedWithGas { output: Vec<u8>, remaining: u64 },
}

impl fmt::Display for VmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfGas => formatter.write_str("out of gas"),
            Self::StackUnderflow => formatter.write_str("stack underflow"),
            Self::StackOverflow => formatter.write_str("stack overflow"),
            Self::InvalidOpcode(opcode) => write!(formatter, "invalid opcode 0x{opcode:02x}"),
            Self::InvalidJump(target) => write!(formatter, "invalid jump destination {target}"),
            Self::CodeOutOfBounds => formatter.write_str("bytecode offset out of bounds"),
            Self::MemoryLimitExceeded => formatter.write_str("memory limit exceeded"),
            Self::ReturnDataOutOfBounds => formatter.write_str("return-data range out of bounds"),
            Self::InsufficientBalance { account, needed, available } => {
                write!(
                    formatter,
                    "insufficient balance for {account:?}: need {needed}, have {available}"
                )
            }
            Self::InvalidNonce { account, expected, actual } => {
                write!(
                    formatter,
                    "invalid nonce for {account:?}: expected {expected}, got {actual}"
                )
            }
            Self::NonceOverflow => formatter.write_str("sender nonce overflow"),
            Self::ValueOverflow => formatter.write_str("balance overflow"),
            Self::CallDepthExceeded => formatter.write_str("call depth limit exceeded"),
            Self::RevertedWithGas { output, .. } => {
                write!(formatter, "execution reverted with {} output bytes", output.len())
            }
        }
    }
}

impl std::error::Error for VmError {}
