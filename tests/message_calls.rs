use minievm::{Address, Evm, Transaction, Word, WorldState};

fn transaction(caller: Address, target: Address, input: Vec<u8>) -> Transaction {
    Transaction { caller, target, value: 0, input, gas_limit: 100_000, nonce: 0 }
}

#[test]
fn calldatasize_reports_bytes_including_partial_words() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // PUSH1 key 0, CALLDATASIZE (value), SSTORE, STOP.
    state.set_code(target, vec![0x60, 0x00, 0x36, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    let result = evm.transact(transaction(caller, target, vec![0xaa; 33]));
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(33));
}

#[test]
fn call_executes_target_and_commits_its_storage() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.set_code(child, vec![0x60, 0x00, 0x60, 0x2a, 0x55, 0x00]);
    // gas, to, value, in_offset, in_size, out_offset, out_size, CALL, STOP.
    state.set_code(
        parent,
        vec![0x61, 0x27, 0x10, 0x60, 3, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0xf1, 0x00],
    );
    let mut evm = Evm::new(state);
    let result = evm.transact(transaction(caller, parent, vec![]));
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(child).unwrap().storage[&Word::ZERO], Word::from_u64(42));
}

#[test]
fn reverted_child_call_does_not_leak_storage() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.set_code(child, vec![0x60, 0x00, 0x60, 7, 0x55, 0x60, 0, 0x60, 0, 0xfd]);
    state.set_code(
        parent,
        vec![0x61, 0x27, 0x10, 0x60, 3, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0xf1, 0x00],
    );
    let mut evm = Evm::new(state);
    let result = evm.transact(transaction(caller, parent, vec![]));
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert!(evm.state.account(child).unwrap().storage.is_empty());
}
