# Integration Tests

Each top-level Rust file in this directory is a Cargo integration-test crate. Keep file
names in `snake_case.rs` and group cases by the behavior they exercise.

| File | Coverage |
| --- | --- |
| `execution.rs` | Storage execution and transaction rollback. |
| `environment.rs` | Block environment values and memory round trips. |
| `byte_opcode.rs` | BYTE behavior. |
| `control_flow.rs` | Jump destination validation. |
| `message_calls.rs` | CALL success, revert, and state isolation. |
| `semantics.rs` | Stack order, copying, return data, gas forwarding, and word boundaries. |
| `advanced_execution.rs` | Arithmetic, logs, transaction boundaries, and receipt behavior. |
| `execution_engine.rs` | Shifts, dynamic memory and copy gas, runtime opcodes, storage clearing, and tracing. |
| `word_arithmetic.rs` | Deterministic 256-bit division, modulo, multiplication, carry and borrow checks. |
| `metadata_and_trace.rs` | Shared opcode metadata, MOD/GT names, limited tracing, and snapshot hooks. |

| `memory_boundaries.rs` | Empty and unaddressable ranges, return-data source bounds, logs and rollback. |
| `call_boundaries.rs` | Local rejection, gas, overflow, stale return data, self-calls and exceptional-child rollback. |
| `stack_memory_opcodes.rs` | PUSH0/MSTORE8 values, gas, stack capacity, memory boundaries and rollback. |
| `diagnostics.rs` | Concrete validation/execution errors and standard error formatting. |

`common/mod.rs` provides setup, transactions and extreme operands for focused tests.
Existing integration file names remain stable; new cases should use behavior-specific
names. The private PUSH decoder also has a unit test across all 32 immediate widths, and
the crate documentation contains an executable example.

Run the full suite with `cargo test`, or a single integration target with
`cargo test --test execution_engine`.
