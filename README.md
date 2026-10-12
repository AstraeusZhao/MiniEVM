# MiniEVM

MiniEVM is a teaching interpreter for a subset of EVM bytecode. It is suitable
for learning and controlled research examples. It is not a fork-compatible Ethereum
client and must not determine real transaction validity, fees, or asset settlement.
Balances are u64; gas is a simplified table without intrinsic/dynamic fork costs;
call depth and memory limits are host resource limits.

## Quick start

Requirements: stable Rust (edition 2024).

```bash
git clone https://github.com/AstraeusZhao/MiniEVM
cd MiniEVM
cargo test
cargo run --example counter
```

## Features

- `Address`, 256-bit `Word`, and deterministic account state
- Account balances, nonce, bytecode, and key-value storage
- Transaction-level checkpoint and rollback boundaries
- Stack limits, configurable linear-memory limits, and gas metering
- PUSH, DUP, SWAP, arithmetic, comparison, bitwise operations, calldata, storage, memory,
  logs, block environment values, jumps, return, and revert
- Structured `ExecutionResult` and error types; tracing types are reserved interfaces
  and are not currently connected to the dispatch loop
- No `unsafe` code, standard Rust formatting, and Clippy-gated CI

## Architecture

```text
Transaction → Evm → Frame → opcode dispatcher
                 │       ├─ stack / Memory / GasMeter
                 │       └─ reserved tracer interfaces
                 └─ WorldState → Account → code / storage / balance
```

See [docs/architecture.md](docs/architecture.md) for module responsibilities and
[docs/opcodes.md](docs/opcodes.md) for opcode coverage and stack conventions.

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

## Development workflow

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Before submitting changes, read [CONTRIBUTING.md](docs/CONTRIBUTING.md),
[CODE_OF_CONDUCT.md](docs/CODE_OF_CONDUCT.md), and [SECURITY.md](docs/SECURITY.md).
MiniEVM is licensed under [LGPL-3.0](LICENSE).

## Status and roadmap

The current release focuses on the execution core. Transaction signing and encoding,
contract creation, additional call-family opcodes, precompiles, fork-specific gas schedules,
and the Ethereum state trie are planned as follow-up modules. Public APIs may change
throughout the `0.x` series.
