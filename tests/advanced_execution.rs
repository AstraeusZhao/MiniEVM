use minievm::{Address, Evm, Transaction, Word, WorldState};

fn tx(caller: Address, target: Address) -> Transaction {
    Transaction { caller, target, value: 0, input: vec![], gas_limit: 10_000, nonce: 0 }
}

#[test]
fn arithmetic_and_bitwise_instructions_persist_the_expected_value() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // ((6 * 7) XOR 3) is stored in storage slot 0.
    state.set_code(target, vec![0x60, 6, 0x60, 7, 0x02, 0x60, 3, 0x18, 0x60, 0x00, 0x55, 0]);
    let mut evm = Evm::new(state);
    assert!(matches!(evm.transact(tx(caller, target)).reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(target).unwrap().storage[&Word::ZERO], Word::from_u64(41));
}

#[test]
fn unsigned_division_handles_words_above_64_bits() {
    let dividend = Word([0x10, 0, 1, 0]);
    let divisor = Word::from_u64(2);

    assert_eq!(dividend.unsigned_div(divisor), Word([8, 0x8000_0000_0000_0000, 0, 0]));
}

#[test]
fn modulo_and_greater_than_cover_zero_and_cross_limb_values() {
    assert_eq!(Word::from_u64(10).unsigned_mod(Word::from_u64(3)), Word::ONE);
    assert_eq!(Word::from_u64(10).unsigned_mod(Word::ZERO), Word::ZERO);
    assert_eq!(Word([0, 1, 0, 0]).unsigned_mod(Word::from_u64(2)), Word::ZERO);
    assert!(Word([0, 1, 0, 0]) > Word::from_u64(u64::MAX));
}

#[test]
fn logs_are_returned_in_successful_receipts() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // LOG1 with offset 0, size 0, topic 0x2a: stack [offset, size, topic].
    state.set_code(target, vec![0x60, 0x2a, 0x60, 0x00, 0x60, 0x00, 0xa1, 0]);
    let mut evm = Evm::new(state);
    let receipt = evm.transact(tx(caller, target));
    assert_eq!(receipt.logs.len(), 1);
    assert_eq!(receipt.logs[0].topics, vec![Word::from_u64(42)]);
}

#[test]
fn invalid_transactions_clear_logs_from_the_previous_receipt() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.set_code(target, vec![0x60, 0x2a, 0x60, 0x00, 0x60, 0x00, 0xa1, 0]);
    let mut evm = Evm::new(state);

    assert_eq!(evm.transact(tx(caller, target)).logs.len(), 1);
    let invalid = evm.transact(tx(caller, target));

    assert!(matches!(invalid.reason, minievm::core::ExitReason::Error));
    assert!(invalid.logs.is_empty());
    assert!(evm.state.logs().is_empty());
}

#[test]
fn transaction_overflow_is_rejected_without_mutating_balances() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.account_mut(target).balance = u64::MAX;
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction { value: 1, ..tx(caller, target) });

    assert!(matches!(result.reason, minievm::core::ExitReason::Error));
    assert_eq!(evm.state.balance_of(caller), 1);
    assert_eq!(evm.state.balance_of(target), u64::MAX);
    assert_eq!(evm.state.account(caller).unwrap().nonce, 0);
}

#[test]
fn nonce_overflow_is_rejected_without_panicking() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.account_mut(caller).nonce = u64::MAX;
    let mut evm = Evm::new(state);

    let result = evm.transact(Transaction { nonce: u64::MAX, ..tx(caller, target) });

    assert!(matches!(result.reason, minievm::core::ExitReason::Error));
    assert_eq!(evm.state.account(caller).unwrap().nonce, u64::MAX);
}

#[test]
fn nonce_is_consumed_by_an_accepted_transaction() {
    let caller = Address::from_last_byte(1);
    let target = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    state.set_code(target, vec![0]);
    let mut evm = Evm::new(state);
    let first = evm.transact(tx(caller, target));
    assert!(matches!(first.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(evm.state.account(caller).unwrap().nonce, 1);
    let repeated = evm.transact(tx(caller, target));
    assert!(matches!(repeated.reason, minievm::core::ExitReason::Error));
}
