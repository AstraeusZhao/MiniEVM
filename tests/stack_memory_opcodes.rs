mod common;

use common::{CONTRACT, run, setup, transaction};
use minievm::{ExitReason, VmError};

#[test]
fn push0_pushes_zero_and_charges_two_gas() {
    let (_, result) = run(vec![0x5f, 0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0xf3], 100);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, [0; 32]);
    assert_eq!(result.gas_used, 17);
    assert_eq!(minievm::opcode::name(0x5f), "PUSH0");
}

#[test]
fn push0_obeys_stack_capacity() {
    let (_, full) = run(vec![0x5f; 1024], 3_000);
    assert_eq!(full.reason, ExitReason::Succeed);
    assert_eq!(full.gas_used, 2_048);
    let (_, overflow) = run(vec![0x5f; 1025], 3_000);
    assert_eq!(overflow.error, Some(VmError::StackOverflow));
    assert_eq!(overflow.gas_used, 3_000);
}

#[test]
fn mstore8_writes_only_low_byte_and_preserves_adjacent_bytes() {
    // Seed memory[31] = 0xff, then store low byte of 0x12ab at memory[1].
    let code =
        vec![0x60, 0xff, 0x60, 0, 0x52, 0x61, 0x12, 0xab, 0x60, 1, 0x53, 0x60, 32, 0x60, 0, 0xf3];
    let (_, result) = run(code, 100);
    let mut expected = [0; 32];
    expected[1] = 0xab;
    expected[31] = 0xff;
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, expected);
    assert_eq!(result.gas_used, 27);
    assert_eq!(minievm::opcode::name(0x53), "MSTORE8");
}

#[test]
fn mstore8_expands_by_words_and_rolls_back_on_limit_or_gas_failure() {
    // A one-byte write at offset 32 expands memory to 64 bytes.
    let (_, result) = run(vec![0x60, 42, 0x60, 32, 0x53, 0], 100);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.gas_used, 15);
    for (limit, gas, error) in
        [(32, 1_000, VmError::MemoryLimitExceeded), (64, 119, VmError::OutOfGas)]
    {
        let mut evm = setup(vec![0x60, 42, 0x60, 0, 0x55, 0x60, 1, 0x60, 32, 0x53, 0]);
        evm.config.max_memory_bytes = limit;
        let result = evm.transact(transaction(gas));
        assert_eq!(result.error, Some(error));
        assert!(evm.state.account(CONTRACT).unwrap().storage.is_empty());
    }
}
