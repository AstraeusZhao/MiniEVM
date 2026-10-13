# MiniEVM

MiniEVM is a hands-on engineering project for Rust developers interested in the execution
engines used by EVM and EVM-compatible L2 systems. It follows the flow from a transaction
into a call frame, through opcode dispatch and state changes, and back into an execution
result. Its core practice areas are the stack, memory, storage, gas, and opcode semantics,
with supporting work on calls, control flow, calldata, logs, and rollback.

MiniEVM is intentionally compact and is not a fork-compatible Ethereum client. Do not
use it to determine real transaction validity, fees, or asset settlement. Balances are
`u64`; gas includes simplified opcode, memory, copy, and log charges but omits intrinsic
gas and fork-specific rules. Call depth and memory limits are runtime resource limits.

## Quick Start

Requirements: stable Rust (edition 2024).

```bash
git clone https://github.com/AstraeusZhao/MiniEVM
cd MiniEVM
cargo test
cargo run --example counter
```

## Core Modules

- **Stack:** 256-bit words, operand order, stack limits, and PUSH/DUP/SWAP behavior
- **Memory:** bounded linear memory, checked ranges, expansion, and return-data copying
- **Storage:** account state, key-value slots, and transaction or call rollback
- **Gas:** per-opcode charging, forwarding, and child-frame settlement
- **Opcodes:** dispatch and semantics for arithmetic, shifts, environment, memory, storage,
  logs, calls, control flow, return, and revert
- **Execution flow:** transaction validation, frame lifecycle, nested calls, and structured
  results and errors
- **Tracing:** opt-in pre-opcode snapshots across parent and nested call frames
- **Rust implementation:** deterministic state, no `unsafe` code, and a focused
  integration-test suite

## Execution Flow

```text
Transaction → Evm → Frame → opcode dispatcher
                 │       ├─ stack / Memory / GasMeter
                 │       └─ optional tracer snapshots
                 └─ WorldState → Account → code / storage / balance
```

See [docs/architecture.md](docs/architecture.md) for the execution path and module map,
[docs/opcodes.md](docs/opcodes.md) for opcode coverage and stack conventions, and
[docs/development.md](docs/development.md) for a practical route through the codebase.

## Example

```rust
use minievm::{Address, Evm, Transaction, WorldState};

let caller = Address::from_last_byte(1);
let contract = Address::from_last_byte(2);
let mut state = WorldState::default();
state.account_mut(caller).balance = 1_000;
state.set_code(contract, vec![0x60, 0x2a, 0x60, 0x00, 0x55, 0x00]);

let result = Evm::new(state).transact(Transaction {
    caller, target: contract, value: 0, input: vec![], gas_limit: 100_000, nonce: 0,
});
assert!(matches!(result.reason, minievm::core::ExitReason::Succeed));
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See the [documentation index](docs/README.md), [Code of Conduct](docs/code-of-conduct.md),
and [Security Policy](docs/security.md) before contributing. MiniEVM is licensed under
[LGPL-3.0](LICENSE).

## Scope and Roadmap

The current release focuses on the execution core. Transaction signing and encoding,
contract creation, additional call-family opcodes, precompiles, fork-specific gas schedules,
and the Ethereum state trie are planned as follow-up modules. Public APIs may change
throughout the `0.x` series.
