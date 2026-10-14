use super::{EvmConfig, bytecode::AnalyzedCode, frame::Frame, instructions};
use crate::{
    core::{BlockEnv, ExecutionResult, ExitReason, Transaction},
    error::VmError,
    gas::{GasMeter, opcode_cost},
    primitives::Address,
    state::WorldState,
    trace::{Step, Tracer},
};

/// Stateful transaction executor.
#[derive(Debug)]
pub struct Evm {
    pub state: WorldState,
    pub config: EvmConfig,
    pub block: BlockEnv,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CallContext {
    pub(super) address: Address,
    pub(super) caller: Address,
    pub(super) value: u64,
    pub(super) depth: u16,
}

impl Evm {
    pub fn new(state: WorldState) -> Self {
        Self { state, config: EvmConfig::default(), block: BlockEnv::default() }
    }
    pub fn with_block_env(mut self, block: BlockEnv) -> Self {
        self.block = block;
        self
    }

    /// Executes one transaction. Validation errors use zero gas; exceptional halts
    /// use the full budget and preserve only the accepted sender nonce.
    pub fn transact(&mut self, tx: Transaction) -> ExecutionResult {
        self.transact_inner(tx, None)
    }

    /// Executes a transaction while reporting the pre-state of each opcode.
    pub fn transact_with_tracer(
        &mut self,
        tx: Transaction,
        tracer: &mut dyn Tracer,
    ) -> ExecutionResult {
        self.transact_inner(tx, Some(tracer))
    }

    fn transact_inner(
        &mut self,
        tx: Transaction,
        mut tracer: Option<&mut dyn Tracer>,
    ) -> ExecutionResult {
        let gas_limit = tx.gas_limit;
        self.state.clear_logs();
        // Read only validation fields; avoid cloning an account's code and storage.
        let (sender_nonce, sender_balance) =
            self.state.account(tx.caller).map_or((0, 0), |sender| (sender.nonce, sender.balance));
        if sender_nonce != tx.nonce {
            return ExecutionResult::failed(
                VmError::InvalidNonce {
                    account: tx.caller,
                    expected: sender_nonce,
                    actual: tx.nonce,
                },
                0,
            );
        }
        if sender_balance < tx.value {
            return ExecutionResult::failed(
                VmError::InsufficientBalance {
                    account: tx.caller,
                    needed: tx.value,
                    available: sender_balance,
                },
                0,
            );
        }
        let Some(next_nonce) = sender_nonce.checked_add(1) else {
            return ExecutionResult::failed(VmError::NonceOverflow, 0);
        };
        let target_balance = if tx.caller == tx.target {
            None
        } else {
            let Some(balance) = self.state.balance_of(tx.target).checked_add(tx.value) else {
                return ExecutionResult::failed(VmError::ValueOverflow, 0);
            };
            Some(balance)
        };
        // Invalid transactions never need a world-state checkpoint.
        let checkpoint = self.state.snapshot();

        self.state.account_mut(tx.caller).nonce = next_nonce;
        if let Some(target_balance) = target_balance {
            self.state.account_mut(tx.caller).balance -= tx.value;
            self.state.account_mut(tx.target).balance = target_balance;
        }
        match self.execute(
            CallContext { address: tx.target, caller: tx.caller, value: tx.value, depth: 0 },
            tx.input,
            gas_limit,
            &mut tracer,
        ) {
            Ok((output, gas_left)) => ExecutionResult {
                reason: ExitReason::Succeed,
                error: None,
                gas_used: gas_limit - gas_left,
                output,
                logs: self.state.logs().to_vec(),
            },
            Err(VmError::RevertedWithGas { output, remaining }) => {
                self.state.restore(checkpoint);
                self.state.account_mut(tx.caller).nonce = next_nonce;
                ExecutionResult {
                    reason: ExitReason::Revert,
                    error: None,
                    gas_used: gas_limit - remaining,
                    output,
                    logs: vec![],
                }
            }
            Err(error) => {
                self.state.restore(checkpoint);
                self.state.account_mut(tx.caller).nonce = next_nonce;
                ExecutionResult::failed(error, gas_limit)
            }
        }
    }

    pub(super) fn execute(
        &mut self,
        context: CallContext,
        input: Vec<u8>,
        gas_limit: u64,
        tracer: &mut Option<&mut dyn Tracer>,
    ) -> Result<(Vec<u8>, u64), VmError> {
        if context.depth >= self.config.max_call_depth {
            return Err(VmError::CallDepthExceeded);
        }
        let code = AnalyzedCode::new(
            self.state
                .account(context.address)
                .map_or_else(Vec::new, |account| account.code.clone()),
        );
        let mut frame = Frame::new(
            context.address,
            context.caller,
            context.value,
            context.depth,
            input,
            self.config.max_memory_bytes,
        );
        let mut gas = GasMeter::new(gas_limit);
        while frame.pc < code.bytes().len() {
            let pc = frame.pc;
            let opcode = code.bytes()[frame.pc];
            if let Some(tracer) = tracer.as_deref_mut()
                && tracer.capture_next()
            {
                tracer.step(Step {
                    address: context.address,
                    depth: context.depth,
                    program_counter: pc,
                    opcode,
                    gas_remaining: gas.remaining(),
                    stack: frame.stack.clone(),
                });
            }
            frame.pc += 1;
            gas.charge(opcode_cost(opcode))?;
            match instructions::dispatch(self, &mut frame, &mut gas, &code, opcode, tracer) {
                Ok(Some(output)) => return Ok((output, gas.remaining())),
                Ok(None) => {}
                Err(error) => return Err(error),
            }
        }
        Ok((vec![], gas.remaining()))
    }
}
