#![allow(dead_code)]

use minievm::{Address, Evm, ExecutionResult, Transaction, WorldState};

pub const CALLER: Address = Address::from_last_byte(1);
pub const CONTRACT: Address = Address::from_last_byte(2);
pub const CHILD: Address = Address::from_last_byte(3);

pub fn setup(code: Vec<u8>) -> Evm {
    let mut state = WorldState::default();
    state.account_mut(CALLER).balance = 100;
    state.set_code(CONTRACT, code);
    Evm::new(state)
}

pub fn transaction(gas_limit: u64) -> Transaction {
    Transaction { caller: CALLER, target: CONTRACT, value: 0, input: vec![], gas_limit, nonce: 0 }
}

pub fn run(code: Vec<u8>, gas_limit: u64) -> (Evm, ExecutionResult) {
    let mut evm = setup(code);
    let result = evm.transact(transaction(gas_limit));
    (evm, result)
}

pub fn push_max(code: &mut Vec<u8>) {
    code.push(0x7f);
    code.extend([0xff; 32]);
}
