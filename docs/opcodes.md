# Opcode Reference

Operands below are listed in **pop order, top of stack first**. `a` is popped before `b`.
For example, `SUB` computes `a - b`; `SSTORE` pops key then value, so code pushes the value
before the key. Prices are this project's simplified base gas, independent of forks.

## Supported Instructions

| Encoding | Instruction | Popped operands / effect | Base gas |
| --- | --- | --- | --- |
| `00` | STOP | Successful empty return | 0 |
| `01` | ADD | a, b → a + b modulo 2²⁵⁶ | 3 |
| `02` | MUL | a, b → a × b modulo 2²⁵⁶ | 5 |
| `03` | SUB | a, b → a − b modulo 2²⁵⁶ | 3 |
| `04` | DIV | a, b → unsigned a / b; zero if b = 0 | 5 |
| `06` | MOD | a, b → unsigned a % b; zero if b = 0 | 5 |
| `10` | LT | a, b → unsigned a < b | 3 |
| `11` | GT | a, b → unsigned a > b | 3 |
| `14` | EQ | a, b → a = b | 3 |
| `15` | ISZERO | a → a = 0 | 3 |
| `16–18` | AND, OR, XOR | a, b → bitwise result | 3 |
| `19` | NOT | a → bitwise inverse | 3 |
| `1a` | BYTE | index, value → selected big-endian byte; zero for index ≥ 32 | 3 |
| `1b–1d` | SHL, SHR, SAR | shift, value → shifted value; SAR sign-extends | 3 |
| `30` | ADDRESS | Push current address | 2 |
| `31` | BALANCE | address → account balance | 100 |
| `33` | CALLER | Push frame caller | 2 |
| `34` | CALLVALUE | Push frame value | 2 |
| `35` | CALLDATALOAD | offset → right-zero-padded 32-byte word | 3 |
| `36` | CALLDATASIZE | Push calldata byte length | 2 |
| `37` | CALLDATACOPY | memory offset, source offset, size | 3 + dynamic |
| `38` | CODESIZE | Push current code byte length | 2 |
| `39` | CODECOPY | memory offset, source offset, size | 3 + dynamic |
| `3d` | RETURNDATASIZE | Push last child output length | 2 |
| `3e` | RETURNDATACOPY | memory offset, data offset, size | 3 + dynamic |
| `41` | COINBASE | Push block beneficiary | 2 |
| `42` | TIMESTAMP | Push block timestamp | 2 |
| `43` | NUMBER | Push block number | 2 |
| `46` | CHAINID | Push chain identifier | 2 |
| `50` | POP | Discard top word | 2 |
| `51` | MLOAD | offset → 32-byte memory word | 3 + expansion |
| `52` | MSTORE | offset, value → write 32 bytes | 3 + expansion |
| `53` | MSTORE8 | offset, value → write lowest 8 bits | 3 + expansion |
| `54` | SLOAD | key → storage value, default zero | 50 |
| `55` | SSTORE | key, value → write slot; remove zero slots | 100 |
| `56` | JUMP | target → jump to executable JUMPDEST | 8 |
| `57` | JUMPI | target, condition → jump if condition ≠ 0 | 10 |
| `58` | PC | Push current instruction's offset | 2 |
| `59` | MSIZE | Push allocated memory size in bytes | 2 |
| `5a` | GAS | Push remaining gas after this instruction's base charge | 2 |
| `5b` | JUMPDEST | Valid jump location | 1 |
| `5f` | PUSH0 | Push zero; no immediate data | 2 |
| `60–7f` | PUSH1–PUSH32 | Push immediate big-endian bytes | 3 |
| `80–8f` | DUP1–DUP16 | Copy word at depth 1–16 | 3 |
| `90–9f` | SWAP1–SWAP16 | Swap top with word 1–16 positions below it | 3 |
| `a0–a4` | LOG0–LOG4 | memory offset, size, topic0 … topicN | 375 + dynamic |
| `f1` | CALL | gas, to, value, input offset/size, output offset/size → success flag | 700 + expansion + child gas |
| `f3` | RETURN | offset, size → successful output bytes | 0 + expansion |
| `fd` | REVERT | offset, size → revert output bytes; retain unused gas | 0 + expansion |

All other bytes cause InvalidOpcode and an exceptional frame halt. Supported instructions
are always enabled; PUSH0 support does not imply compatibility with any particular fork.

## Dynamic Gas

For `w` allocated 32-byte words, `C(w) = 3w + floor(w² / 512)`. Expansion charges the
increase in C, rounded up to words. Copying adds `3 × ceil(size / 32)` gas. Logs add
`375 × topic_count + 8 × data_size` gas. CALL reserves at most 63/64 of remaining gas.

Intrinsic transaction gas, value stipends, access warming, storage refunds, and fork
schedules are omitted. These prices must not be used for real fee estimation.

## Boundary Rules

- Stack size is at most 1,024 words. Missing operands and excessive pushes fail.
- Oversized shifts return zero, or all ones for a negative SAR operand.
- Missing PUSH bytes are padded on the right; bytecode execution then reaches its end.
- Jumps cannot land inside PUSH immediate data. JUMPI ignores its target when condition is zero.
- Zero-length memory ranges ignore their memory offset and do not expand memory.
- CALLDATA/CODE copies zero-pad beyond the source. RETURNDATACOPY instead requires
  `data_offset + size <= return_data.len()`, including when size is zero.
- CALL depth/balance rejection returns zero, clears return data, and spends no child gas.
  Recipient balance overflow is a local rejection in the `u64` balance model.
- Child REVERT returns bytes and unused gas. Exceptional children consume forwarded gas.
  Both restore child state and logs; successful CALL only overwrites actual returned bytes.

## Walkthrough

`60 2a 5f 55 00` stores 42 in slot zero. Stacks below are shown **bottom to top**.

| PC | Instruction | Stack afterward | Observable effect |
| --- | --- | --- | --- |
| 0 | PUSH1 0x2a | [42] | 3 gas |
| 2 | PUSH0 | [42, 0] | 2 gas |
| 3 | SSTORE | [] | storage[0] = 42; 100 gas |
| 4 | STOP | [] | Commit; total 105 gas |

See [test coverage](../tests/README.md) and the runnable
[trace example](../examples/trace.rs) for memory and execution observations.

Names, base prices, minimum stack height, stack changes and immediate lengths are
exposed through `minievm::opcode::metadata`. Pricing and diagnostics share this table.
