# Audit Remediation

This note summarizes MiniEVM execution-engine findings and their status in the current
implementation. It describes project code only; it does not assert that repository policy,
CI, or remote security settings have been independently verified.

| Area | Current status |
| --- | --- |
| 256-bit word ordering and division | Numeric ordering compares high limbs first; division is covered by boundary cases. |
| Opcode stack order | CALL, copy, log, arithmetic, and storage handlers use documented top-first operands. |
| Memory allocation | Memory ranges are checked before copying; calldata and return-data copies avoid temporary buffers. |
| Revert and nonce handling | REVERT preserves remaining gas; accepted transactions consume the sender nonce on failure. |
| Bytecode boundaries | BYTE rejects oversized indices; truncated PUSH data is zero-padded; jumps validate executable JUMPDEST positions. |
| CALL return data | Only returned bytes overwrite the requested output region. |
| Gas model | Memory expansion, word copying, and log data are charged; the model remains simplified and fork-independent. |
| Tracing | `transact_with_tracer` reports pre-opcode state for top-level and nested frames. |

These behaviors are implementation claims, not a substitute for the regression suite. See
the [test layout](../tests/README.md), [opcode reference](opcodes.md), and
[security scan scope](security-scan-scope.md).
