use minievm::{Address, Evm, Transaction, WorldState};

#[test]
fn jump_rejects_jumpdest_bytes_embedded_in_push_data() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // PUSH1 0x5b embeds a byte that looks like JUMPDEST at offset 1. The subsequent
    // PUSH1 1 / JUMP must reject it because immediate data is not executable code.
    state.set_code(target, vec![0x60, 0x5b, 0x50, 0x60, 0x01, 0x56]);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target,
        value: 0,
        input: vec![],
        gas_limit: 100,
        nonce: 0,
    });
    assert!(matches!(result.reason, minievm::core::ExitReason::Error));
}
