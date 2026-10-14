# Development Guide

Start with `Evm::transact`, follow the frame execution loop, then inspect one instruction
and its stack, memory, gas and state effects. The [engineering plan](engineering-plan.md)
sets the current scope and future candidates.

## Conventions

Use the pinned Rust 1.97.1 toolchain and Edition 2024. Use UTF-8, LF endings, snake_case
files and rustfmt; `.editorconfig` and `.gitattributes` encode repository conventions.
Public APIs should describe units, limits, conversions and failure behavior in rustdoc.

Keep helpers with their execution responsibility. Instructions live under
`src/vm/instructions/`; the dispatcher stays visible in `mod.rs`. Add instructions in
opcode order and update their entry in `opcode/metadata.rs` and reference documentation.
Raw bytecode tests should state whether comments list push order or pop order.

## Error Handling

Return checked VmError values for bytecode errors. Receipts retain validation and
exceptional errors in `ExecutionResult::error`; success and REVERT use None. REVERT output
is returned separately. Invalid transactions consume zero gas and no nonce. Accepted
transactions retain their nonce on revert or error, while contract state is restored.

CALL precondition rejection is local: push zero and preserve the child budget. An
exceptional child that actually started consumes its forwarded gas. Preserve memory
expansion charges and clear stale return data in both cases.

## Tests

For semantics changes, assert actual returned bytes, storage, balances, logs, gas and/or
tracer state. A successful exit alone cannot verify an environment value or memory read.
Use shared setup in `tests/common/mod.rs`, keeping bytecode and assertions in each case.
Private decoding details may use unit tests, such as exact truncated PUSH values.

New instructions need normal and boundary cases plus meaningful failure/rollback checks.
See the [test directory guide](../tests/README.md). Fuzzing and numerical differential
testing are future candidates; retain failing inputs or seeds when adding them.

## Local Checks

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo run --locked --example counter
cargo run --locked --example trace
```

`cargo test` includes the crate-level rustdoc example. `cargo doc` generates the API index
under `target/doc/minievm/`. The tracked `.github/workflows/ci.yml` runs formatting, Clippy, tests and documentation
builds on push and pull requests using Rust 1.97.1.

## Review and Performance

Explain the behavior changed, why, tests run, and any public API changes. Keep performance
claims tied to reproducible benchmarks. Full-state checkpoints, repeated code analysis,
MLOAD allocation and tracer stack snapshots remain candidates for measured optimization.
Dependency additions should state their purpose and update the
[dependency baseline](dependency-baseline.md).
