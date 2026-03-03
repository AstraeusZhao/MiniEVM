use crate::primitives::Address;

/// Immutable block metadata visible to a transaction execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockEnv {
    pub number: u64,
    pub timestamp: u64,
    pub chain_id: u64,
    pub coinbase: Address,
}
