# Opcode coverage

| Family | Instructions |
| --- | --- |
| Halt | `STOP`, `RETURN`, `REVERT` |
| Arithmetic | `ADD`, `MUL`, `SUB`, `DIV`, `BYTE` |
| Comparison and bitwise | `LT`, `EQ`, `ISZERO`, `AND`, `OR`, `XOR`, `NOT` |
| Environment | `ADDRESS`, `BALANCE`, `CALLER`, `CALLVALUE`, `CALLDATALOAD`, `CALLDATASIZE`, `CALLDATACOPY`, `CODESIZE`, `CODECOPY`, `RETURNDATASIZE`, `RETURNDATACOPY`, `TIMESTAMP`, `NUMBER`, `CHAINID` |
| Stack | `POP`, `PUSH1..PUSH32`, `DUP1..DUP16`, `SWAP1..SWAP16` |
| Memory | `MLOAD`, `MSTORE` |
| Storage | `SLOAD`, `SSTORE` |
| Logs | `LOG0..LOG4` |
| Calls | `CALL` |
| Control flow | `JUMP`, `JUMPI`, `JUMPDEST` |

Operands are popped top-first. For example `SSTORE` pops `key` then `value`; therefore bytecode pushes `value` before `key`. Every jump target must point exactly at a `JUMPDEST` byte.
