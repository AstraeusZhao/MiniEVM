# Architecture

MiniEVM is organized as a compact, hands-on implementation of the execution-engine flow
used by EVM and EVM-compatible L2 systems. The main path is `Transaction` ->
`Evm::transact` -> `Frame` -> opcode dispatch -> `ExecutionResult`. Following this path
shows how transaction checks, execution context, opcode effects, and state commit or
rollback fit together.

The five core practice areas are:

- **Stack:** `Frame` owns the operand stack and enforces its limit and pop ordering.
- **Memory:** `memory::Memory` provides bounded, checked linear-memory expansion.
- **Storage:** `WorldState` and `Account` hold deterministic account data and key-value
  slots; transaction and nested-call checkpoints control rollback.
- **Gas:** `gas::GasMeter` tracks frame budgets, charges instructions, and settles child
  frame gas. Memory expansion, copy size, and log data add simplified dynamic costs.
- **Opcodes:** `vm::instructions` implements the instruction semantics and connects them
  to stack, memory, storage, gas, and call state.

`Evm::transact` is the transaction boundary. It validates the sender nonce, snapshots
`WorldState`, transfers call value, executes code in a `Frame`, and commits only a
successful frame. Successful receipts include emitted logs. Execution-local state such as
the operand stack, linear memory, program counter, calldata, and caller is not persisted
between transactions.

Supporting modules cover calldata, bytecode analysis for jump validation, logs, and block
environment values. `Evm::transact_with_tracer` optionally reports each frame's address
and depth, plus the pre-instruction PC, opcode, remaining gas, and stack.

The public API is intentionally narrow: create a `WorldState`, attach accounts and code,
create `Evm`, then submit `Transaction` values. The implementation is useful for studying
and extending this execution path; its simplified protocol model is not a drop-in Ethereum
or L2 client.
