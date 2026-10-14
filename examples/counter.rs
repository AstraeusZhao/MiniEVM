use minievm::{Address, Evm, ExitReason, Transaction, Word, WorldState};

fn main() {
    let caller = Address::from_last_byte(1);
    let counter = Address::from_last_byte(2);
    let mut state = WorldState::default();
    // PUSH0, SLOAD, PUSH1 1, ADD, PUSH0, SSTORE, STOP.
    state.set_code(counter, vec![0x5f, 0x54, 0x60, 1, 0x01, 0x5f, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    println!("counter = 0");
    for nonce in 0..2 {
        let result = evm.transact(Transaction {
            caller,
            target: counter,
            value: 0,
            input: vec![],
            gas_limit: 10_000,
            nonce,
        });
        assert_eq!(result.reason, ExitReason::Succeed, "{:?}", result.error);
        let count = evm.state.account(counter).unwrap().storage[&Word::ZERO].low_u64();
        assert_eq!(count, nonce + 1);
        println!("nonce={nonce}, counter={count}, gas_used={}", result.gas_used);
    }
}
