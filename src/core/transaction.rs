use crate::primitives::Address;

/// An externally supplied message call.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub caller: Address,
    pub target: Address,
    pub value: u64,
    pub input: Vec<u8>,
    pub gas_limit: u64,
    pub nonce: u64,
}
