# Development guide

## Project conventions

Use Rust stable and edition 2024. Public types need rustdoc comments. Keep opcode stack ordering explicit at the dispatch site, and put semantic regressions in `tests/` with raw bytecode that demonstrates the exact behavior.

## Error handling

The VM does not panic on untrusted bytecode. Invalid stack access, invalid jumps, exhausted gas and memory-limit breaches must map to `VmError`. A transaction error restores the state snapshot.

## Test strategy

Run unit and integration tests locally. New instructions should receive ordinary-path, boundary, and transaction-rollback tests. Fuzzing can target `Evm::transact` because it accepts owned byte buffers and has no external dependencies.
