//! MiniEVM is a modular Rust execution engine for EVM-style bytecode.

pub mod core;
pub mod error;
pub mod gas;
pub mod memory;
pub mod opcode;
pub mod primitives;
pub mod state;
pub mod trace;
pub mod vm;

pub use core::{BlockEnv, ExecutionResult, Transaction};
pub use error::VmError;
pub use primitives::{Address, Word};
pub use state::{Account, WorldState};
pub use vm::{Evm, EvmConfig};
