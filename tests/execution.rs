use minievm::{Address, Evm, Transaction, Word, WorldState};

#[test]
fn stores_and_loads_a_word() {
    let contract = Address::from_last_byte(2);
    let caller = Address::from_last_byte(1);
    // PUSH1 value, PUSH1 key, SSTORE, PUSH1 key, SLOAD, STOP
    let code = vec![0x60, 0x2a, 0x60, 0x01, 0x55, 0x60, 0x01, 0x54, 0x00];
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 10;
    state.set_code(contract, code);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target: contract,
        value: 0,
        input: vec![],
        gas_limit: 1_000,
        nonce: 0,
    });
    assert_eq!(result.reason, minievm::core::ExitReason::Succeed);
    assert_eq!(
        evm.state.account(contract).unwrap().storage[&Word::from_u64(1)],
        Word::from_u64(42)
    );
}

#[test]
fn revert_restores_value_transfer() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 7;
    state.set_code(target, vec![0x60, 0x00, 0x60, 0x00, 0xfd]);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target,
        value: 3,
        input: vec![],
        gas_limit: 100,
        nonce: 0,
    });
    assert_eq!(result.reason, minievm::core::ExitReason::Revert);
    assert_eq!(evm.state.balance_of(caller), 7);
    assert_eq!(evm.state.balance_of(target), 0);
}
