mod common;

use common::{CALLER, CONTRACT, run, setup, transaction};
use minievm::{ExitReason, VmError};

#[test]
fn validation_failures_report_details_without_consuming_nonce_or_gas() {
    let mut evm = setup(vec![0]);
    let result = evm.transact(minievm::Transaction { nonce: 1, ..transaction(100) });
    assert_eq!(
        result.error,
        Some(VmError::InvalidNonce { account: CALLER, expected: 0, actual: 1 })
    );
    assert_eq!(result.gas_used, 0);
    let result = evm.transact(minievm::Transaction { value: 101, ..transaction(100) });
    assert_eq!(
        result.error,
        Some(VmError::InsufficientBalance { account: CALLER, needed: 101, available: 100 })
    );
    evm.state.account_mut(CONTRACT).balance = u64::MAX;
    let result = evm.transact(minievm::Transaction { value: 1, ..transaction(100) });
    assert_eq!(result.error, Some(VmError::ValueOverflow));
    assert_eq!(evm.state.account(CALLER).unwrap().nonce, 0);
    evm.state.account_mut(CALLER).nonce = u64::MAX;
    let result = evm.transact(minievm::Transaction { nonce: u64::MAX, ..transaction(100) });
    assert_eq!(result.error, Some(VmError::NonceOverflow));
    assert_eq!(result.gas_used, 0);
}

#[test]
fn exceptional_halts_report_cause_and_restore_contract_state() {
    for (tail, expected) in [
        (vec![0xfe], VmError::InvalidOpcode(0xfe)),
        (vec![0x50], VmError::StackUnderflow),
        (vec![0x60, 0, 0x56], VmError::InvalidJump(0)),
    ] {
        let mut code = vec![0x60, 42, 0x60, 0, 0x55];
        code.extend(tail);
        let (evm, result) = run(code, 1_000);
        assert_eq!(result.reason, ExitReason::Error);
        assert_eq!(result.error, Some(expected));
        assert_eq!(result.gas_used, 1_000);
        assert!(evm.state.account(CONTRACT).unwrap().storage.is_empty());
        assert_eq!(evm.state.account(CALLER).unwrap().nonce, 1);
    }
}

#[test]
fn vm_error_supports_standard_error_and_readable_display() {
    let error: &dyn std::error::Error = &VmError::InvalidOpcode(0xfe);
    assert_eq!(error.to_string(), "invalid opcode 0xfe");
}
