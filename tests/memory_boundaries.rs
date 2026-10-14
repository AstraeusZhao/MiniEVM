mod common;

use common::{CONTRACT, push_max, run};
use minievm::{ExitReason, VmError};

#[test]
fn empty_return_and_revert_ignore_unaddressable_offset() {
    for (opcode, reason) in [(0xf3, ExitReason::Succeed), (0xfd, ExitReason::Revert)] {
        let mut code = vec![0x60, 0]; // size
        push_max(&mut code); // offset
        code.push(opcode);
        let (_, result) = run(code, 100);
        assert_eq!(result.reason, reason);
        assert_eq!(result.error, None);
        assert_eq!(result.gas_used, 6);
        assert!(result.output.is_empty());
    }
}

#[test]
fn empty_copies_ignore_memory_offset_and_do_not_expand() {
    for opcode in [0x37, 0x39, 0x3e] {
        // Zero size and valid source offset; enormous destination offset.
        let mut code = vec![0x60, 0, 0x60, 0];
        push_max(&mut code);
        code.extend([opcode, 0x59, 0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0xf3]);
        let (_, result) = run(code, 100);
        assert_eq!(result.reason, ExitReason::Succeed);
        assert_eq!(result.output, [0; 32]); // MSIZE before MSTORE is zero.
        assert_eq!(result.gas_used, 29);
    }
}

#[test]
fn empty_return_data_copy_still_checks_source_offset() {
    let mut code = vec![0x60, 0];
    push_max(&mut code); // source offset is beyond empty return data
    code.extend([0x60, 0, 0x3e]);
    let (_, result) = run(code, 100);
    assert_eq!(result.error, Some(VmError::ReturnDataOutOfBounds));
    assert_eq!(result.gas_used, 100);
}

#[test]
fn empty_log_ignores_offset_and_commits_topics() {
    let mut code = vec![0x60, 42, 0x60, 0];
    push_max(&mut code);
    code.extend([0xa1, 0]);
    let (_, result) = run(code, 1_000);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.gas_used, 759);
    assert_eq!(result.logs.len(), 1);
    assert_eq!(result.logs[0].topics, vec![minievm::Word::from_u64(42)]);
    assert!(result.logs[0].data.is_empty());
}

#[test]
fn nonempty_unaddressable_range_rolls_back_storage() {
    let mut code = vec![0x60, 42, 0x60, 0, 0x55, 0x60, 1];
    push_max(&mut code);
    code.push(0xf3);
    let (evm, result) = run(code, 1_000);
    assert_eq!(result.error, Some(VmError::MemoryLimitExceeded));
    assert!(evm.state.account(CONTRACT).unwrap().storage.is_empty());
}
