/// Returns the simplified base price from shared opcode metadata.
/// Unsupported bytes retain a nominal base price of 3 before exceptional halt.
pub const fn opcode_cost(opcode: u8) -> u64 {
    match crate::opcode::metadata(opcode) {
        Some(info) => info.base_gas,
        None => 3,
    }
}
