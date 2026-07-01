# Contributing to MiniEVM

Thank you for contributing. Search existing issues before opening a new one. Changes
to opcode semantics, gas, or state transitions must include minimal reproducing
bytecode and tests.

## Local checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Submission requirements

- Keep one pull request focused on one clear concern and describe behavioral and
  compatibility impact.
- Update the README or `docs/` when changing public APIs, opcodes, or error semantics.
- Every bug fix needs a regression test that fails before the fix.
- Do not commit `target/`, keys, real RPC credentials, or generated artifacts.

By contributing, you confirm that you may license your contribution under LGPL-3.0.
