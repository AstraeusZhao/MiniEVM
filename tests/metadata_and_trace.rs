mod common;

use common::{run, setup, transaction};
use minievm::{
    ExitReason, VmError, Word, opcode,
    trace::{LimitedTracer, Tracer, VecTracer},
};

#[test]
fn new_arithmetic_opcodes_report_names_and_values() {
    assert_eq!(opcode::name(0x06), "MOD");
    assert_eq!(opcode::name(0x11), "GT");
    let code = vec![0x60, 3, 0x60, 10, 0x06, 0x60, 0, 0x55, 0];
    let (evm, result) = run(code, 1_000);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(evm.state.account(common::CONTRACT).unwrap().storage[&Word::ZERO], Word::ONE);
    let code = vec![0x60, 3, 0x60, 10, 0x11, 0x60, 0, 0x55, 0];
    let (evm, result) = run(code, 1_000);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(evm.state.account(common::CONTRACT).unwrap().storage[&Word::ZERO], Word::ONE);
}

#[test]
fn arithmetic_opcodes_handle_zero_equal_operands_and_out_of_gas_rollback() {
    for (opcode, divisor, dividend, expected, base) in [
        (0x06, 0, 10, 0, 5),
        (0x06, 3, 10, 1, 5),
        (0x11, 10, 10, 0, 3),
        (0x11, 10, 3, 0, 3),
        (0x11, 3, 10, 1, 3),
    ] {
        let code = vec![0x60, divisor, 0x60, dividend, opcode, 0x5f, 0x52, 0x60, 32, 0x5f, 0xf3];
        let (_, result) = run(code, 100);
        assert_eq!(result.reason, ExitReason::Succeed);
        assert_eq!(result.output, Word::from_u64(expected).to_be_bytes());
        assert_eq!(result.gas_used, 19 + base);
    }
    // A preceding write must roll back when MOD has insufficient gas.
    let code = vec![0x60, 42, 0x5f, 0x55, 0x60, 3, 0x60, 10, 0x06];
    let (evm, result) = run(code, 115);
    assert_eq!(result.error, Some(VmError::OutOfGas));
    assert!(evm.state.account(common::CONTRACT).unwrap().storage.is_empty());
}

#[test]
fn limited_tracer_records_a_prefix_and_marks_truncation() {
    let mut evm = setup(vec![0x5f; 8]);
    let mut tracer = LimitedTracer::new(3);
    let result = evm.transact_with_tracer(transaction(100), &mut tracer);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(tracer.steps().len(), 3);
    assert!(tracer.truncated());
    assert_eq!(tracer.steps()[0].opcode, 0x5f);
}

#[test]
fn zero_limit_limited_tracer_records_nothing_but_marks_truncation() {
    let mut evm = setup(vec![0]);
    let mut tracer = LimitedTracer::new(0);
    let result = evm.transact_with_tracer(transaction(100), &mut tracer);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert!(tracer.steps().is_empty());
    assert!(tracer.truncated());
}

#[test]
fn metadata_agrees_with_dispatch_support_and_stack_requirements() {
    for opcode in 0..=255 {
        let (_, result) = run(vec![opcode], 1_000);
        match opcode::metadata(opcode) {
            None => {
                assert_eq!(result.error, Some(VmError::InvalidOpcode(opcode)), "0x{opcode:02x}");
                assert_eq!(opcode::name(opcode), "INVALID");
            }
            Some(info) => {
                assert_eq!(minievm::gas::opcode_cost(opcode), info.base_gas);
                assert_eq!(opcode::name(opcode), info.name);
                if info.required_stack > 0 {
                    assert_eq!(result.error, Some(VmError::StackUnderflow), "0x{opcode:02x}");
                } else {
                    assert_eq!(result.reason, ExitReason::Succeed, "0x{opcode:02x}");
                }
            }
        }
    }
    assert_eq!(opcode::metadata(0x7f).unwrap().immediate_bytes, 32);
    assert_eq!(opcode::metadata(0x8f).unwrap().required_stack, 16);
    assert_eq!(opcode::metadata(0x9f).unwrap().required_stack, 17);
    assert_eq!(opcode::metadata(0xa4).unwrap().stack_delta, -6);
}

#[test]
fn limited_tracing_preserves_execution_and_nested_frame_prefix() {
    let code = vec![0x5f, 0x5f, 0x5f, 0x5f, 0x5f, 0x60, 3, 0x61, 0x03, 0xe8, 0xf1, 0];
    let mut evm = setup(code);
    evm.state.set_code(common::CHILD, vec![0x60, 42, 0x5f, 0x55, 0]);
    let mut reference = minievm::Evm::new(evm.state.clone());
    let mut full = VecTracer::default();
    let expected = reference.transact_with_tracer(transaction(10_000), &mut full);
    let mut limited = LimitedTracer::new(10);
    let actual = evm.transact_with_tracer(transaction(10_000), &mut limited);
    assert_eq!(actual, expected);
    assert_eq!(limited.steps(), &full.steps[..10]);
    assert!(limited.steps().iter().any(|step| step.depth == 1));
    assert!(limited.truncated());
    assert_eq!(evm.state.account(common::CHILD).unwrap().storage[&Word::ZERO], Word::from_u64(42));
}

#[test]
fn trace_capture_hook_can_skip_events_without_receiving_snapshots() {
    struct SkipTracer(usize);
    impl Tracer for SkipTracer {
        fn capture_next(&mut self) -> bool {
            self.0 += 1;
            false
        }
        fn step(&mut self, _: minievm::trace::Step) {
            panic!("skipped instructions must not dispatch snapshots");
        }
    }
    let mut evm = setup(vec![0x5f, 0x50, 0]);
    let mut tracer = SkipTracer(0);
    let result = evm.transact_with_tracer(transaction(100), &mut tracer);
    assert_eq!(result.reason, ExitReason::Succeed);
    assert_eq!(tracer.0, 3);
}
