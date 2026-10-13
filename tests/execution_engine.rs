use minievm::{Address, BlockEnv, Evm, Transaction, Word, WorldState, trace::VecTracer};

fn setup(code: Vec<u8>) -> (Evm, Address, Address) {
    let caller = Address::from_last_byte(1);
    let contract = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.set_code(contract, code);
    let evm = Evm::new(state);
    (evm, caller, contract)
}

fn transact(
    evm: &mut Evm,
    caller: Address,
    contract: Address,
    gas_limit: u64,
) -> minievm::ExecutionResult {
    evm.transact(Transaction {
        caller,
        target: contract,
        value: 0,
        input: vec![],
        gas_limit,
        nonce: 0,
    })
}

#[test]
fn shifts_cover_limb_boundaries_and_signed_fill() {
    assert_eq!(Word::ONE.shift_left(Word::from_u64(65)), Word([0, 2, 0, 0]));
    assert_eq!(
        Word([0, 0, 0, 1 << 63]).shift_arithmetic_right(Word::ONE),
        Word([0, 0, 0, 0xc000_0000_0000_0000])
    );
    assert_eq!(Word::ONE.shift_right(Word([0, 1, 0, 0])), Word::ZERO);

    let (mut evm, caller, contract) = setup(vec![0x60, 1, 0x60, 8, 0x1b, 0x60, 0, 0x55, 0]);
    assert!(matches!(
        transact(&mut evm, caller, contract, 1_000).reason,
        minievm::core::ExitReason::Succeed
    ));
    assert_eq!(evm.state.account(contract).unwrap().storage[&Word::ZERO], Word::from_u64(256));
}

#[test]
fn memory_expansion_and_copy_charge_dynamic_gas() {
    let (mut evm, caller, contract) = setup(vec![0x60, 0x2a, 0x60, 0, 0x52, 0]);
    let result = transact(&mut evm, caller, contract, 100);
    assert_eq!(result.gas_used, 12);

    let (mut evm, caller, contract) = setup(vec![0x60, 32, 0x60, 0, 0x60, 0, 0x37, 0]);
    let result = evm.transact(Transaction {
        caller,
        target: contract,
        value: 0,
        input: vec![0xaa; 32],
        gas_limit: 100,
        nonce: 0,
    });
    assert_eq!(result.gas_used, 18);
}

#[test]
fn runtime_opcodes_report_coinbase_pc_msize_and_remaining_gas() {
    let coinbase = Address::from_last_byte(9);
    let (mut evm, caller, contract) =
        setup(vec![0x41, 0x60, 1, 0x55, 0x60, 0x2a, 0x60, 0, 0x52, 0x59, 0x60, 2, 0x55, 0x00]);
    evm.block = BlockEnv { coinbase, ..BlockEnv::default() };
    let result = transact(&mut evm, caller, contract, 1_000);
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(
        evm.state.account(contract).unwrap().storage[&Word::from_u64(1)],
        Word::from_be_slice(&coinbase.0)
    );
    assert_eq!(
        evm.state.account(contract).unwrap().storage[&Word::from_u64(2)],
        Word::from_u64(32)
    );

    let (mut evm, caller, contract) = setup(vec![0x58, 0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0xf3]);
    let result = transact(&mut evm, caller, contract, 100);
    assert_eq!(result.output, Word::ZERO.to_be_bytes());

    let (mut evm, caller, contract) = setup(vec![0x5a, 0x60, 0, 0x55, 0]);
    transact(&mut evm, caller, contract, 1_000);
    assert_eq!(evm.state.account(contract).unwrap().storage[&Word::ZERO], Word::from_u64(998));
}

#[test]
fn storing_zero_removes_the_storage_slot() {
    let (mut evm, caller, contract) = setup(vec![0x60, 0, 0x60, 0, 0x55, 0]);
    evm.state.account_mut(contract).storage.insert(Word::ZERO, Word::from_u64(1));
    let result = transact(&mut evm, caller, contract, 1_000);
    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert!(!evm.state.account(contract).unwrap().storage.contains_key(&Word::ZERO));
}

#[test]
fn tracer_reports_pre_opcode_state_and_nested_frame_identity() {
    let caller = Address::from_last_byte(1);
    let parent = Address::from_last_byte(2);
    let child = Address::from_last_byte(3);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 100;
    state.set_code(child, vec![0x60, 0x2a, 0x00]);
    state.set_code(
        parent,
        vec![0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x60, 3, 0x61, 0x27, 0x10, 0xf1, 0x00],
    );
    let mut evm = Evm::new(state);
    let mut tracer = VecTracer::default();
    let result = evm.transact_with_tracer(
        Transaction {
            caller,
            target: parent,
            value: 0,
            input: vec![],
            gas_limit: 100_000,
            nonce: 0,
        },
        &mut tracer,
    );

    assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
    assert_eq!(tracer.steps[0].address, parent);
    assert_eq!(tracer.steps[0].depth, 0);
    assert_eq!(tracer.steps[0].program_counter, 0);
    let child_step = tracer.steps.iter().find(|step| step.address == child).unwrap();
    assert_eq!(child_step.depth, 1);
    assert_eq!(child_step.program_counter, 0);
}
