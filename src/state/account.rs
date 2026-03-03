use crate::primitives::Word;
use std::collections::BTreeMap;

/// Persistent data associated with an address.
#[derive(Debug, Clone, Default)]
pub struct Account {
    pub nonce: u64,
    pub balance: u64,
    pub code: Vec<u8>,
    pub storage: BTreeMap<Word, Word>,
}
