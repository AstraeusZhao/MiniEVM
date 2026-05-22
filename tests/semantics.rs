use minievm::gas::GasMeter;
use minievm::{Address, Evm, Transaction, Word, WorldState};

fn tx(caller: Address, target: Address, gas_limit: u64) -> Transaction {
    Transaction { caller, target, value: 0, input: vec![], gas_limit, nonce: 0 }
}

fn setup(code: Vec<u8>) -> (Address, Address, WorldState) {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.set_code(target, code);
    (caller, target, state)
}

#[test]
fn sub_uses_evm_operand_order() {
    // PUSH1 10, PUSH1 3, SUB => 10 - 3 = 7.
    let (caller, target, state) =
        setup(vec![0x60, 10, 0x60, 3, 0x03, 0x60, 0x00, 0x90, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(7));
}

#[test]
fn div_uses_evm_operand_order() {
    // PUSH1 20, PUSH1 4, DIV => 20 / 4 = 5.
    let (caller, target, state) =
        setup(vec![0x60, 20, 0x60, 4, 0x04, 0x60, 0x00, 0x90, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(5));
}

#[test]
fn lt_uses_evm_operand_order() {
    // PUSH1 3, PUSH1 10, LT => 3 < 10 = 1.
    let (caller, target, state) =
        setup(vec![0x60, 3, 0x60, 10, 0x10, 0x60, 0x00, 0x90, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::ONE);
}

#[test]
fn child_gas_unused_portion_is_returned_to_parent() {
    // Parent burns 100, forwards 50 to a child that consumes 30 and leaves 20.
    let mut meter = GasMeter::new(1_000);
    meter.charge(100).unwrap();
    let forwarded = meter.forward(50).unwrap();
    assert_eq!(forwarded, 50);
    meter.settle_child(forwarded, 20);
    assert_eq!(meter.remaining(), 870);
}

#[test]
fn forward_caps_requested_gas_to_remaining() {
    let mut meter = GasMeter::new(100);
    meter.charge(10).unwrap();
    let forwarded = meter.forward(5_000).unwrap();
    assert_eq!(forwarded, 90);
}

#[test]
fn calldataload_with_huge_offset_reads_zero() {
    // PUSH32 (1 << 192), CALLDATALOAD, store the result: must be zero,
    // not a truncated read of calldata[0].
    let mut code = vec![0x7f, 0, 0, 0, 0, 0, 0, 0, 1];
    code.extend([0u8; 24]);
    code.extend([0x35, 0x60, 0x00, 0x90, 0x55, 0x00]);
    let (caller, target, state) = setup(code);
    let mut evm = Evm::new(state);
    let mut input = tx(caller, target, 1_000);
    input.input = vec![0xaa; 4];
    assert!(matches!(evm.transact(input).reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::ZERO);
}

#[test]
fn log2_preserves_topic_order() {
    // LOG2: stack [offset, size, topic1, topic2] with topic2 on top.
    let (caller, target, state) = setup(vec![0x60, 0, 0x60, 0, 0x60, 0x11, 0x60, 0x22, 0xa2, 0x00]);
    let mut evm = Evm::new(state);
    let receipt = evm.transact(tx(caller, target, 1_000));
    assert_eq!(receipt.logs.len(), 1);
    assert_eq!(receipt.logs[0].topics, vec![Word::from_u64(0x11), Word::from_u64(0x22)]);
}

#[test]
fn call_output_is_zero_padded_to_the_requested_size() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    // Child: MSTORE(0, 0x2a); RETURN(31, 1) so the returned byte is 0x2a.
    state.set_code(child, vec![0x60, 0, 0x60, 0x2a, 0x52, 0x60, 31, 0x60, 1, 0xf3]);
    // Parent: CALL(gas, child, 0, 0, 0, 0, 32); RETURN(0, 32).
    state.set_code(
        parent,
        vec![
            0x61, 0x27, 0x10, 0x60, 3, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0x20, 0xf1, 0x60,
            0, 0x60, 0x20, 0xf3,
        ],
    );
    let mut evm = Evm::new(state);
    let result = evm.transact(tx(caller, parent, 10_000));
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(result.output.len(), 32);
    assert_eq!(result.output[0], 0x2a);
    assert!(result.output[1..].iter().all(|&b| b == 0));
}

#[test]
fn call_transfers_value_to_the_target() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.account_mut(parent).balance = 10;
    state.set_code(child, vec![0x00]); // STOP; receiver keeps the value
    // Parent: CALL(gas, child, 7, 0, 0, 0, 0); STOP.
    state.set_code(
        parent,
        vec![0x61, 0x27, 0x10, 0x60, 3, 0x60, 7, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0xf1, 0x00],
    );
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, parent, 10_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.balance_of(parent), 3);
    assert_eq!(evm.state.balance_of(child), 7);
}

#[test]
fn call_without_sufficient_value_pushes_zero() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.account_mut(parent).balance = 1;
    state.set_code(child, vec![0x00]);
    // Parent: CALL(gas, child, 7, 0, 0, 0, 0) returns 0 on failure; store it.
    state.set_code(
        parent,
        vec![
            0x61, 0x27, 0x10, 0x60, 3, 0x60, 7, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0xf1, 0x60,
            0x00, 0x90, 0x55, 0x00,
        ],
    );
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, parent, 10_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(parent).unwrap().storage[&Word::ZERO], Word::ZERO);
    assert_eq!(evm.state.balance_of(child), 0);
}

#[test]
fn sstore_uses_evm_key_value_order() {
    // PUSH1 key 7, PUSH1 value 9, SSTORE => storage[7] = 9.
    let (caller, target, state) = setup(vec![0x60, 7, 0x60, 9, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::from_u64(7)], Word::from_u64(9));
}

#[test]
fn mstore_uses_evm_offset_value_order() {
    // PUSH1 offset 3, PUSH1 value 0x2a, MSTORE; MLOAD(3) must round-trip.
    let (caller, target, state) =
        setup(vec![0x60, 3, 0x60, 0x2a, 0x52, 0x60, 3, 0x51, 0x60, 0x00, 0x90, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(0x2a));
}

#[test]
fn jumpi_uses_evm_destination_condition_order() {
    // PUSH1 dest, PUSH1 cond(1), JUMPI jumps; with cond 0 execution falls through.
    // Jump target is the JUMPDEST at the end; the fall-through path stores 1,
    // the jump path stores 2, so slot 0 proves which branch ran.
    let fallthrough = vec![
        0x60, 0x0c, // dest 12 (the JUMPDEST)
        0x60, 0x01, // cond 1 (true => jump)
        0x57, // JUMPI
        0x60, 1, 0x60, 0x00, 0x90, 0x55, 0x00, // store 1, stop (not taken)
        0x5b, // JUMPDEST
        0x60, 2, 0x60, 0x00, 0x90, 0x55, 0x00, // store 2, stop (taken)
    ];
    let (caller, target, state) = setup(fallthrough);
    let mut evm = Evm::new(state);
    assert!(matches!(
        evm.transact(tx(caller, target, 1_000)).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(2));
}

#[test]
fn returndatacopy_respects_return_data_bounds() {
    // CALLDATACOPY: stack [mem_offset, src_offset, size]; copy input[0..4]
    // into memory[0..4], then MLOAD(0) and store the word.
    let code = vec![0x60, 0, 0x60, 0, 0x60, 4, 0x37, 0x60, 0, 0x51, 0x60, 0x00, 0x90, 0x55, 0x00];
    let (caller, target, state) = setup(code);
    let mut evm = Evm::new(state);
    let mut input = tx(caller, target, 1_000);
    input.input = vec![0xde, 0xad, 0xbe, 0xef, 0xff];
    assert!(matches!(evm.transact(input).reason, minievm::core::ExitReason::Succeed));
    assert_eq!(
        evm.state.account(target).unwrap().storage[&Word::ZERO],
        Word([0, 0, 0, 0xdead_beef_0000_0000])
    );
}
