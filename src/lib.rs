//! MiniEVM is a modular Rust execution engine for EVM-style bytecode.
//!
//! Submit raw bytecode and inspect the resulting persistent state:
//!
//! ```
//! use minievm::{Address, Evm, ExitReason, Transaction, Word, WorldState};
//!
//! let caller = Address::from_last_byte(1);
//! let contract = Address::from_last_byte(2);
//! let mut state = WorldState::default();
//! // PUSH1 42, PUSH0, SSTORE, STOP: storage[0] = 42.
//! state.set_code(contract, vec![0x60, 42, 0x5f, 0x55, 0x00]);
//! let mut evm = Evm::new(state);
//! let result = evm.transact(Transaction {
//!     caller, target: contract, value: 0, input: vec![], gas_limit: 1_000, nonce: 0,
//! });
//! assert_eq!(result.reason, ExitReason::Succeed);
//! assert_eq!(result.error, None);
//! assert_eq!(evm.state.account(contract).unwrap().storage[&Word::ZERO], Word::from_u64(42));
//! ```

pub mod core;
pub mod error;
pub mod gas;
pub mod memory;
pub mod opcode;
pub mod primitives;
pub mod state;
pub mod trace;
pub mod vm;

pub use core::{BlockEnv, ExecutionResult, ExitReason, Transaction};
pub use error::VmError;
pub use primitives::{Address, Word};
pub use state::{Account, WorldState};
pub use vm::{Evm, EvmConfig};
