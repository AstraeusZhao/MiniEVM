//! Shared instruction metadata for pricing and diagnostics.

/// Static information for an instruction supported by this interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeInfo {
    pub name: &'static str,
    /// Base gas only; memory, copying, logs, and child execution add dynamic costs.
    pub base_gas: u64,
    /// Minimum stack height required before the instruction.
    pub required_stack: u8,
    /// Stack height change after a successful instruction (outputs minus inputs).
    pub stack_delta: i8,
    /// Encoded immediate length; nonzero only for PUSH1 through PUSH32.
    pub immediate_bytes: u8,
}

const fn info(
    name: &'static str,
    base_gas: u64,
    required_stack: u8,
    stack_delta: i8,
    immediate_bytes: u8,
) -> OpcodeInfo {
    OpcodeInfo { name, base_gas, required_stack, stack_delta, immediate_bytes }
}

/// Returns metadata for a supported opcode, or None for invalid/unsupported bytes.
pub const fn metadata(opcode: u8) -> Option<OpcodeInfo> {
    Some(match opcode {
        0x00 => info("STOP", 0, 0, 0, 0),
        0x01 => info("ADD", 3, 2, -1, 0),
        0x02 => info("MUL", 5, 2, -1, 0),
        0x03 => info("SUB", 3, 2, -1, 0),
        0x04 => info("DIV", 5, 2, -1, 0),
        0x06 => info("MOD", 5, 2, -1, 0),
        0x10 => info("LT", 3, 2, -1, 0),
        0x11 => info("GT", 3, 2, -1, 0),
        0x14 => info("EQ", 3, 2, -1, 0),
        0x15 => info("ISZERO", 3, 1, 0, 0),
        0x16 => info("AND", 3, 2, -1, 0),
        0x17 => info("OR", 3, 2, -1, 0),
        0x18 => info("XOR", 3, 2, -1, 0),
        0x19 => info("NOT", 3, 1, 0, 0),
        0x1a => info("BYTE", 3, 2, -1, 0),
        0x1b => info("SHL", 3, 2, -1, 0),
        0x1c => info("SHR", 3, 2, -1, 0),
        0x1d => info("SAR", 3, 2, -1, 0),
        0x30 => info("ADDRESS", 2, 0, 1, 0),
        0x31 => info("BALANCE", 100, 1, 0, 0),
        0x33 => info("CALLER", 2, 0, 1, 0),
        0x34 => info("CALLVALUE", 2, 0, 1, 0),
        0x35 => info("CALLDATALOAD", 3, 1, 0, 0),
        0x36 => info("CALLDATASIZE", 2, 0, 1, 0),
        0x37 => info("CALLDATACOPY", 3, 3, -3, 0),
        0x38 => info("CODESIZE", 2, 0, 1, 0),
        0x39 => info("CODECOPY", 3, 3, -3, 0),
        0x3d => info("RETURNDATASIZE", 2, 0, 1, 0),
        0x3e => info("RETURNDATACOPY", 3, 3, -3, 0),
        0x41 => info("COINBASE", 2, 0, 1, 0),
        0x42 => info("TIMESTAMP", 2, 0, 1, 0),
        0x43 => info("NUMBER", 2, 0, 1, 0),
        0x46 => info("CHAINID", 2, 0, 1, 0),
        0x50 => info("POP", 2, 1, -1, 0),
        0x51 => info("MLOAD", 3, 1, 0, 0),
        0x52 => info("MSTORE", 3, 2, -2, 0),
        0x53 => info("MSTORE8", 3, 2, -2, 0),
        0x54 => info("SLOAD", 50, 1, 0, 0),
        0x55 => info("SSTORE", 100, 2, -2, 0),
        0x56 => info("JUMP", 8, 1, -1, 0),
        0x57 => info("JUMPI", 10, 2, -2, 0),
        0x58 => info("PC", 2, 0, 1, 0),
        0x59 => info("MSIZE", 2, 0, 1, 0),
        0x5a => info("GAS", 2, 0, 1, 0),
        0x5b => info("JUMPDEST", 1, 0, 0, 0),
        0x5f => info("PUSH0", 2, 0, 1, 0),
        0x60..=0x7f => info("PUSH", 3, 0, 1, opcode - 0x5f),
        0x80..=0x8f => info("DUP", 3, opcode - 0x7f, 1, 0),
        0x90..=0x9f => info("SWAP", 3, opcode - 0x8f + 1, 0, 0),
        0xa0..=0xa4 => info("LOG", 375, opcode - 0xa0 + 2, -((opcode - 0xa0 + 2) as i8), 0),
        0xf1 => info("CALL", 700, 7, -6, 0),
        0xf3 => info("RETURN", 0, 2, -2, 0),
        0xfd => info("REVERT", 0, 2, -2, 0),
        _ => return None,
    })
}
