mod common;

use common::{run, setup, transaction};
use minievm::{Address, BlockEnv, ExitReason, Word};

#[test]
fn environment_opcodes_read_the_configured_block() {
    // Store each environment value in a separate output word.
    let code = vec![
        0x42, 0x60, 0, 0x52, 0x43, 0x60, 32, 0x52, 0x46, 0x60, 64, 0x52, 0x60, 96, 0x60, 0, 0xf3,
    ];
    let mut evm = setup(code);
    evm.block =
        BlockEnv { number: 99, timestamp: 1_700_000_000, chain_id: 31337, coinbase: Address::ZERO };
    let result = evm.transact(transaction(100));
    assert_eq!(result.reason, ExitReason::Succeed);
    let expected: Vec<u8> = [1_700_000_000, 99, 31337]
        .into_iter()
        .flat_map(|value| Word::from_u64(value).to_be_bytes())
        .collect();
    assert_eq!(result.output, expected);
}

#[test]
fn mstore_and_mload_round_trip() {
    // Write at 0, load it, then write the loaded value at 32 and return it.
    let code =
        vec![0x60, 42, 0x60, 0, 0x52, 0x60, 0, 0x51, 0x60, 32, 0x52, 0x60, 32, 0x60, 32, 0xf3];
    let (_, result) = run(code, 100);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, Word::from_u64(42).to_be_bytes());
}
