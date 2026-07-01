# Architecture

`Evm::transact` is the transaction boundary. It validates the sender nonce, snapshots
`WorldState`, transfers call value, executes code in one `Frame`, and commits only a
successful frame. Successful receipts include emitted logs. `WorldState` holds deterministic
`BTreeMap` account data. Execution state is deliberately local: operand stack, linear memory,
program counter, calldata, and caller are not persisted between transactions.

`gas::GasMeter` owns remaining resource budget; `memory::Memory` performs checked expansion against a caller-supplied cap; `trace::Tracer` is an opt-in observer boundary. The VM is decomposed into `engine` (transaction/frame lifecycle), `frame` (ephemeral stack and context), `instructions` (opcode semantics), and `calldata` (data-word extraction).

The public API is intentionally narrow: create a `WorldState`, attach accounts/code, create `Evm`, then submit `Transaction` values.
