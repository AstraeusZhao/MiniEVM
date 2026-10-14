mod common;

use common::{CALLER, CHILD, CONTRACT, push_max, setup, transaction};
use minievm::{ExitReason, Word};

fn call_code(value: u8) -> Vec<u8> {
    // Push in reverse pop order: output size/offset, input size/offset, value, to, gas.
    let mut code =
        vec![0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, value, 0x60, 3, 0x61, 0x03, 0xe8, 0xf1];
    // Return the CALL status and RETURNDATASIZE as two words.
    code.extend([0x60, 0, 0x52, 0x3d, 0x60, 32, 0x52, 0x60, 64, 0x60, 0, 0xf3]);
    code
}

#[test]
fn call_rejections_preserve_gas_balances_and_parent_execution() {
    for rejection in ["depth", "balance", "overflow"] {
        let value = u8::from(rejection != "depth");
        let mut evm = setup(call_code(value));
        if rejection == "depth" {
            evm.config.max_call_depth = 1;
        } else if rejection == "overflow" {
            evm.state.account_mut(CONTRACT).balance = 1;
            evm.state.account_mut(CHILD).balance = u64::MAX;
        }
        // This child would leave evidence if it executed.
        evm.state.set_code(CHILD, vec![0x60, 42, 0x60, 0, 0x55, 0]);
        let before_parent = evm.state.balance_of(CONTRACT);
        let before_child = evm.state.balance_of(CHILD);
        let result = evm.transact(transaction(10_000));
        assert_eq!(result.reason, ExitReason::Succeed, "{rejection}");
        assert_eq!(result.error, None);
        assert_eq!(result.output, [0; 64]);
        assert_eq!(result.gas_used, 747, "rejected child must not consume 1,000 gas");
        assert_eq!(evm.state.balance_of(CONTRACT), before_parent);
        assert_eq!(evm.state.balance_of(CHILD), before_child);
        assert!(evm.state.account(CHILD).unwrap().storage.is_empty());
        assert_eq!(evm.state.account(CALLER).unwrap().nonce, 1);
    }
}

#[test]
fn empty_call_ranges_ignore_unaddressable_offsets() {
    let mut code = vec![0x60, 0]; // output size
    push_max(&mut code);
    code.extend([0x60, 0]); // input size
    push_max(&mut code);
    code.extend([0x60, 0, 0x60, 3, 0x61, 0x03, 0xe8, 0xf1]);
    code.extend([0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0xf3]);
    let mut evm = setup(code);
    let result = evm.transact(transaction(10_000));
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, Word::ONE.to_be_bytes());
    assert_eq!(result.gas_used, 736);
}

#[test]
fn exceptional_child_spends_forwarded_gas_and_rolls_back_value_and_logs() {
    let mut evm = setup(call_code(1));
    evm.state.account_mut(CONTRACT).balance = 2;
    evm.state.set_code(CHILD, vec![0x60, 0, 0x60, 0, 0xa0, 0xfe]);
    let result = evm.transact(transaction(10_000));
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, [0; 64]);
    assert_eq!(result.gas_used, 1_747);
    assert_eq!(evm.state.balance_of(CONTRACT), 2);
    assert_eq!(evm.state.balance_of(CHILD), 0);
    assert!(result.logs.is_empty());
}

#[test]
fn rejected_call_clears_previous_return_data() {
    // First child returns one byte; a second CALL is rejected for insufficient balance.
    let first = call_code(0);
    let mut code = first[..16].to_vec();
    code.push(0x50); // discard success
    code.extend(call_code(1));
    let mut evm = setup(code);
    evm.state.set_code(CHILD, vec![0x60, 42, 0x60, 0, 0x52, 0x60, 1, 0x60, 31, 0xf3]);
    let result = evm.transact(transaction(10_000));
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, [0; 64]);
}

#[test]
fn self_call_at_max_balance_preserves_balance() {
    let mut code = call_code(1);
    code[11] = 2; // self address
    let mut evm = setup(code);
    evm.state.account_mut(CONTRACT).balance = u64::MAX;
    evm.config.max_call_depth = 2;
    let result = evm.transact(transaction(10_000));
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(&result.output[..32], &Word::ONE.to_be_bytes());
    assert_eq!(evm.state.balance_of(CONTRACT), u64::MAX);
}
