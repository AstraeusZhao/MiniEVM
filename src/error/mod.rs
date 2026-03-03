use crate::primitives::Address;

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
    ValueOverflow,
    CallDepthExceeded,
    Reverted(Vec<u8>),
}
