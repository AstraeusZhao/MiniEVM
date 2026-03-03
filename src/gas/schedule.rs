/// The table is intentionally indexed by raw opcode so that adding an instruction
/// requires updating one obvious dispatch table as well as the interpreter.
pub const fn opcode_cost(opcode: u8) -> u64 {
    let mut costs = [3u64; 256];
    costs[0x00] = 0;
    costs[0x01] = 3;
    costs[0x02] = 5;
    costs[0x03] = 3;
    costs[0x04] = 5;
    costs[0x10] = 3;
    costs[0x14] = 3;
    costs[0x15] = 3;
    costs[0x16] = 3;
    costs[0x17] = 3;
    costs[0x18] = 3;
    costs[0x1a] = 0;
    costs[0x30] = 2;
    costs[0x31] = 100;
    costs[0x33] = 2;
    costs[0x34] = 2;
    costs[0x35] = 3;
    costs[0x36] = 2;
    costs[0x37] = 3;
    costs[0x38] = 2;
    costs[0x39] = 3;
    costs[0x3d] = 2;
    costs[0x3e] = 3;
    costs[0x42] = 2;
    costs[0x43] = 2;
    costs[0x46] = 2;
    costs[0x50] = 2;
    costs[0x51] = 3;
    costs[0x52] = 3;
    costs[0x54] = 50;
    costs[0x55] = 100;
    costs[0x56] = 8;
    costs[0x57] = 10;
    costs[0x5b] = 1;
    costs[0xf1] = 700;
    costs[0xf3] = 0;
    costs[0xfd] = 0;
    costs[opcode as usize]
}
