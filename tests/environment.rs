use minievm::{Address, BlockEnv, Evm, Transaction, WorldState};

#[test]
fn environment_opcodes_read_the_configured_block() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.set_code(target, vec![0x42, 0x43, 0x46, 0x00]);
    let block =
        BlockEnv { number: 99, timestamp: 1_700_000_000, chain_id: 31337, coinbase: Address::ZERO };
    let mut evm = Evm::new(state).with_block_env(block);
    let result = evm.transact(Transaction {
        caller,
        target,
        value: 0,
        input: vec![],
        gas_limit: 100,
        nonce: 0,
    });
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
}

#[test]
fn mstore_and_mload_round_trip() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // PUSH1 offset, PUSH1 value, MSTORE, PUSH1 offset, MLOAD, STOP
    state.set_code(target, vec![0x60, 0x00, 0x60, 0x2a, 0x52, 0x60, 0x00, 0x51, 0x00]);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target,
        value: 0,
        input: vec![],
        gas_limit: 100,
        nonce: 0,
    });
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
}
