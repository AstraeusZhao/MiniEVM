use minievm::{Address, Evm, ExitReason, Transaction, WorldState, opcode, trace::LimitedTracer};

fn main() {
    let caller = Address::from_last_byte(1);
    let contract = Address::from_last_byte(2);
    let mut state = WorldState::default();
    // Write one byte and return it: demonstrates PUSH0 and MSTORE8.
    state.set_code(contract, vec![0x60, 42, 0x5f, 0x53, 0x60, 1, 0x5f, 0xf3]);
    let mut evm = Evm::new(state);
    let mut tracer = LimitedTracer::new(100);
    let result = evm.transact_with_tracer(
        Transaction { caller, target: contract, value: 0, input: vec![], gas_limit: 100, nonce: 0 },
        &mut tracer,
    );
    for step in tracer.steps() {
        println!(
            "depth={} pc={} opcode=0x{:02x} {} gas={} stack={:?}",
            step.depth,
            step.program_counter,
            step.opcode,
            opcode::name(step.opcode),
            step.gas_remaining,
            step.stack,
        );
    }
    assert!(!tracer.truncated());
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(result.output, vec![42]);
    println!("output={:02x?}, gas_used={}", result.output, result.gas_used);
}
