use minievm::{Address, Evm, Transaction, Word, WorldState};

#[test]
fn byte_returns_the_selected_big_endian_byte() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // PUSH1 0xab, PUSH1 31, BYTE, PUSH1 key 0, SWAP1, SSTORE, STOP.
    state.set_code(target, vec![0x60, 0xab, 0x60, 31, 0x1a, 0x60, 0, 0x90, 0x55, 0]);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target,
        value: 0,
        input: vec![],
        gas_limit: 1_000,
        nonce: 0,
    });
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(0xab));
}
