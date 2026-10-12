use minievm::core::ExitReason;
use minievm::{Address, Evm, Transaction, Word, WorldState};
fn run(code: Vec<u8>, input: Vec<u8>) -> (Evm, minievm::core::ExecutionResult) {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.set_code(target, code);
    let mut evm = Evm::new(state);
    let receipt =
        evm.transact(Transaction { caller, target, value: 0, input, gas_limit: 100_000, nonce: 0 });
    (evm, receipt)
}
#[test]
fn numeric_order_and_division_cross_limb_boundaries() {
    for limb in 1..4 {
        let mut a = [0; 4];
        a[limb] = 1;
        assert!(Word(a) > Word::from_u64(u64::MAX));
    }
    assert_eq!(Word([0, 1, 0, 0]).unsigned_div(Word::from_u64(2)), Word::from_u64(1 << 63));
    let max = Word([u64::MAX; 4]);
    assert_eq!(max.unsigned_div(Word([0, 0, 0, 1 << 63])), Word::ONE);
}
#[test]
fn arithmetic_pops_operand_zero_first() {
    for (op, a, b, want) in [(0x03, 3, 10, 7), (0x04, 4, 20, 5), (0x10, 10, 3, 1)] {
        let (evm, r) = run(vec![0x60, a, 0x60, b, op, 0x60, 0, 0x55, 0], vec![]);
        assert_eq!(r.reason, ExitReason::Succeed);
        assert_eq!(
            evm.state.account(Address::from_last_byte(2)).unwrap().storage[&Word::ZERO],
            Word::from_u64(want)
        );
    }
}
#[test]
fn copies_pop_memory_offset_first_and_pad_source() {
    let (_, r) = run(vec![0x60, 4, 0x60, 1, 0x60, 2, 0x37, 0x60, 6, 0x60, 0, 0xf3], vec![1, 2, 3]);
    assert_eq!(r.output, vec![0, 0, 2, 3, 0, 0]);
}
#[test]
fn codecopy_uses_top_first_offsets() {
    let (_, r) = run(vec![0x60, 2, 0x60, 0, 0x60, 0, 0x39, 0x60, 2, 0x60, 0, 0xf3], vec![]);
    assert_eq!(r.output, vec![0x60, 2]);
}
#[test]
fn logs_pop_offset_size_before_topics() {
    let (_, r) = run(vec![0x60, 0x22, 0x60, 0x11, 0x60, 0, 0x60, 0, 0xa2, 0], vec![]);
    assert_eq!(r.logs[0].topics, vec![Word::from_u64(0x11), Word::from_u64(0x22)]);
}
#[test]
fn enormous_copy_is_rejected_before_allocation() {
    let mut code = vec![0x7f];
    code.extend([0xff; 32]);
    code.extend([0x60, 0, 0x60, 0, 0x37]);
    let (_, r) = run(code, vec![]);
    assert_eq!(r.reason, ExitReason::Error);
}
#[test]
fn high_byte_index_does_not_truncate() {
    let mut code = vec![0x60, 0xab, 0x7f];
    code.extend([0xff; 32]);
    code.extend([0x1a, 0x60, 0, 0x55, 0]);
    let (evm, r) = run(code, vec![]);
    assert_eq!(r.reason, ExitReason::Succeed);
    assert_eq!(
        evm.state.account(Address::from_last_byte(2)).unwrap().storage[&Word::ZERO],
        Word::ZERO
    );
}
#[test]
fn truncated_push_pads_immediate_on_the_right() {
    let (_, r) = run(vec![0x61, 0xab], vec![]);
    assert_eq!(r.reason, ExitReason::Succeed);
}
#[test]
fn revert_preserves_nonce_and_remaining_gas() {
    let (evm, r) = run(vec![0x60, 0, 0x60, 0, 0xfd], vec![]);
    assert_eq!(r.reason, ExitReason::Revert);
    assert_eq!(r.gas_used, 6);
    assert_eq!(evm.state.account(Address::from_last_byte(1)).unwrap().nonce, 1);
}
#[test]
fn exceptional_halt_also_consumes_nonce() {
    let (evm, r) = run(vec![0xfe], vec![]);
    assert_eq!(r.reason, ExitReason::Error);
    assert_eq!(evm.state.account(Address::from_last_byte(1)).unwrap().nonce, 1);
}
#[test]
fn call_return_preserves_existing_tail_and_child_revert_gas() {
    for revert in [false, true] {
        let caller = Address::from_last_byte(1);
        let parent = Address::from_last_byte(2);
        let child = Address::from_last_byte(3);
        let mut state = WorldState::default();
        state.account_mut(caller).balance = 100;
        state.set_code(
            child,
            vec![0x60, 0x2a, 0x60, 0, 0x52, 0x60, 1, 0x60, 31, if revert { 0xfd } else { 0xf3 }],
        );
        state.set_code(
            parent,
            vec![
                0x60, 0xff, 0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 3,
                0x61, 0x27, 0x10, 0xf1, 0x50, 0x60, 32, 0x60, 0, 0xf3,
            ],
        );
        let mut evm = Evm::new(state);
        let r = evm.transact(Transaction {
            caller,
            target: parent,
            value: 0,
            input: vec![],
            gas_limit: 100_000,
            nonce: 0,
        });
        assert_eq!(r.reason, ExitReason::Succeed);
        assert_eq!(r.output[0], 0x2a);
        assert_eq!(r.output[31], 0xff);
        assert!(r.gas_used < 1000);
    }
}
#[test]
fn eip150_forwarding_leaves_one_sixty_fourth() {
    let mut gas = minievm::gas::GasMeter::new(6400);
    assert_eq!(gas.forward(u64::MAX).unwrap(), 6300);
    assert_eq!(gas.remaining(), 100);
}
