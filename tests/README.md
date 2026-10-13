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
| `execution_engine.rs` | Shifts, dynamic memory and copy Gas, runtime opcodes, storage clearing, and tracing. |

Run the full suite with `cargo test`, or a single integration target with
`cargo test --test execution_engine`.
