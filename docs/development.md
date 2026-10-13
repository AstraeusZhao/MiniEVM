# Development Guide

MiniEVM is a hands-on Rust codebase for working through the execution-engine flow used by
EVM and EVM-compatible L2 systems. A useful route through the implementation is to follow
a transaction into `Evm::transact`, inspect how `Frame` holds execution-local state, then
trace an opcode through `instructions::dispatch` into the stack, memory, gas meter, or
world state.

The five core areas to explore are stack, memory, storage, gas, and opcode semantics.
Transaction handling, nested calls, control flow, logs, and rollback show how those pieces
work together in a complete execution path.

## Conventions

Use the pinned Rust toolchain and edition 2024. Add rustdoc comments to public types. Keep
opcode stack ordering explicit in the dispatcher, and use raw bytecode in integration tests
to show the exact behavior being changed.

## Error Handling

Untrusted bytecode must not cause a panic. Invalid stack access, invalid jumps, exhausted
gas, and memory-limit breaches must return `VmError`. A failed execution restores contract
state while retaining the accepted transaction nonce.

## Tests

New instructions should receive ordinary-path, boundary, and transaction-rollback tests.
Use raw bytecode in integration tests to make stack ordering and observable state changes
explicit. See the [test directory guide](../tests/README.md) for the current file layout.
Fuzzing can target `Evm::transact`, which accepts owned byte buffers and has no external
dependencies.

The current implementation deliberately simplifies several protocol details. Keep those
limits in view when extending the engine: matching an opcode in isolation does not make
execution fork-compatible, since transaction rules and gas schedules also vary by fork.

## Local Checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
