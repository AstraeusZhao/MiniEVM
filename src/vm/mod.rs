//! EVM orchestration. Implementation details are split by execution responsibility.

mod bytecode;
mod calldata;
mod engine;
mod frame;
mod instructions;

pub use engine::Evm;

/// Runtime limits governing a single EVM instance.
#[derive(Debug, Clone)]
pub struct EvmConfig {
    pub max_call_depth: u16,
    pub max_memory_bytes: usize,
}

impl Default for EvmConfig {
    fn default() -> Self {
        Self { max_call_depth: 64, max_memory_bytes: 16 * 1024 * 1024 }
    }
}
