# Opcode Reference

| Family | Instructions |
| --- | --- |
| Halt | `STOP`, `RETURN`, `REVERT` |
| Arithmetic | `ADD`, `MUL`, `SUB`, `DIV`, `BYTE`, `SHL`, `SHR`, `SAR` |
| Comparison and bitwise | `LT`, `EQ`, `ISZERO`, `AND`, `OR`, `XOR`, `NOT` |
| Environment | `ADDRESS`, `BALANCE`, `CALLER`, `CALLVALUE`, `CALLDATALOAD`, `CALLDATASIZE`, `CALLDATACOPY`, `CODESIZE`, `CODECOPY`, `RETURNDATASIZE`, `RETURNDATACOPY`, `COINBASE`, `TIMESTAMP`, `NUMBER`, `CHAINID`, `PC`, `MSIZE`, `GAS` |
| Stack | `POP`, `PUSH1..PUSH32`, `DUP1..DUP16`, `SWAP1..SWAP16` |
| Memory | `MLOAD`, `MSTORE` |
| Storage | `SLOAD`, `SSTORE` |
| Logs | `LOG0..LOG4` |
| Calls | `CALL` |
| Control flow | `JUMP`, `JUMPI`, `JUMPDEST` |

Operands are popped top-first. For example `SSTORE` pops `key` then `value`; therefore bytecode pushes `value` before `key`. Every jump target must point exactly at a `JUMPDEST` byte.

Gas charges use a simplified schedule. Memory expansion follows a word-based linear and
quadratic cost, copy operations charge per word, and logs charge for topics and data. The
schedule is not tied to an Ethereum fork and is not suitable for fee estimation.
