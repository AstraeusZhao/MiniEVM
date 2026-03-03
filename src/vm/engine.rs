use super::{EvmConfig, bytecode::AnalyzedCode, frame::Frame, instructions};
use crate::{
    core::{BlockEnv, ExecutionResult, ExitReason, Transaction},
    error::VmError,
    gas::{GasMeter, opcode_cost},
    primitives::Address,
    state::WorldState,
};

/// Stateful transaction executor.
#[derive(Debug)]
pub struct Evm {
    pub state: WorldState,
    pub config: EvmConfig,
    pub block: BlockEnv,
}

impl Evm {
    pub fn new(state: WorldState) -> Self {
        Self { state, config: EvmConfig::default(), block: BlockEnv::default() }
    }
    pub fn with_block_env(mut self, block: BlockEnv) -> Self {
        self.block = block;
        self
    }

    pub fn transact(&mut self, tx: Transaction) -> ExecutionResult {
        let gas_limit = tx.gas_limit;
        self.state.clear_logs();
        let checkpoint = self.state.snapshot();
        let sender = self.state.account(tx.caller).cloned().unwrap_or_default();
        if sender.nonce != tx.nonce || sender.balance < tx.value {
            return ExecutionResult {
                reason: ExitReason::Error,
                gas_used: 0,
                output: vec![],
                logs: vec![],
            };
        }
        let next_nonce = match sender.nonce.checked_add(1) {
            Some(nonce) => nonce,
            None => {
                return ExecutionResult {
                    reason: ExitReason::Error,
                    gas_used: 0,
                    output: vec![],
                    logs: vec![],
                };
            }
        };
        let target_balance = if tx.caller == tx.target {
            None
        } else {
            match self.state.balance_of(tx.target).checked_add(tx.value) {
                Some(balance) => Some(balance),
                None => {
                    return ExecutionResult {
                        reason: ExitReason::Error,
                        gas_used: 0,
                        output: vec![],
                        logs: vec![],
                    };
                }
            }
        };

        self.state.account_mut(tx.caller).nonce = next_nonce;
        if let Some(target_balance) = target_balance {
            self.state.account_mut(tx.caller).balance -= tx.value;
            self.state.account_mut(tx.target).balance = target_balance;
        }
        match self.execute(tx.target, tx.caller, tx.value, tx.input, gas_limit, 0) {
            Ok((output, gas_left)) => ExecutionResult {
                reason: ExitReason::Succeed,
                gas_used: gas_limit - gas_left,
                output,
                logs: self.state.logs().to_vec(),
            },
            Err(VmError::Reverted(output)) => {
                self.state.restore(checkpoint);
                ExecutionResult {
                    reason: ExitReason::Revert,
                    gas_used: gas_limit,
                    output,
                    logs: vec![],
                }
            }
            Err(_) => {
                self.state.restore(checkpoint);
                ExecutionResult {
                    reason: ExitReason::Error,
                    gas_used: gas_limit,
                    output: vec![],
                    logs: vec![],
                }
            }
        }
    }

    pub(super) fn execute(
        &mut self,
        address: Address,
        caller: Address,
        call_value: u64,
        input: Vec<u8>,
        gas_limit: u64,
        depth: u16,
    ) -> Result<(Vec<u8>, u64), VmError> {
        if depth >= self.config.max_call_depth {
            return Err(VmError::CallDepthExceeded);
        }
        let code = AnalyzedCode::new(
            self.state.account(address).map_or_else(Vec::new, |account| account.code.clone()),
        );
        let mut frame =
            Frame::new(address, caller, call_value, input, self.config.max_memory_bytes);
        let mut gas = GasMeter::new(gas_limit);
        while frame.pc < code.bytes().len() {
            let opcode = code.bytes()[frame.pc];
            frame.pc += 1;
            gas.charge(opcode_cost(opcode))?;
            if let Some(output) =
                instructions::dispatch(self, &mut frame, &mut gas, address, depth, &code, opcode)?
            {
                return Ok((output, gas.remaining()));
            }
        }
        Ok((vec![], gas.remaining()))
    }
}
