use super::{Account, LogEntry};
use crate::primitives::Address;
use std::collections::BTreeMap;

/// Deterministic in-memory account store and current transaction log buffer.
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    accounts: BTreeMap<Address, Account>,
    logs: Vec<LogEntry>,
}

impl WorldState {
    pub fn account(&self, address: Address) -> Option<&Account> {
        self.accounts.get(&address)
    }
    pub fn account_mut(&mut self, address: Address) -> &mut Account {
        self.accounts.entry(address).or_default()
    }
    pub fn set_code(&mut self, address: Address, code: Vec<u8>) {
        self.account_mut(address).code = code;
    }
    pub fn balance_of(&self, address: Address) -> u64 {
        self.account(address).map_or(0, |account| account.balance)
    }
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
    pub fn restore(&mut self, snapshot: Self) {
        *self = snapshot;
    }
    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }
    pub fn emit_log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
    }
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
}
