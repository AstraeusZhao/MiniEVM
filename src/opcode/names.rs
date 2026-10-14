/// Returns the diagnostic family name, or INVALID for unsupported bytes.
pub const fn name(opcode: u8) -> &'static str {
    match super::metadata(opcode) {
        Some(info) => info.name,
        None => "INVALID",
    }
}
