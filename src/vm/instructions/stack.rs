//! Immediate stack operands.

use super::super::frame::Frame;
use crate::{error::VmError, primitives::Word};

pub(super) fn push_immediate(frame: &mut Frame, code: &[u8], opcode: u8) -> Result<(), VmError> {
    let width = (opcode - 0x5f) as usize;
    let end = frame.pc.checked_add(width).ok_or(VmError::CodeOutOfBounds)?;
    let mut bytes = [0; 32];
    let available = code.len().saturating_sub(frame.pc).min(width);
    bytes[..available].copy_from_slice(&code[frame.pc..frame.pc + available]);
    frame.push(Word::from_be_slice(&bytes[..width]))?;
    frame.pc = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Address;

    #[test]
    fn truncated_push_right_pads_each_immediate_width() {
        for width in 1..=32 {
            let mut frame = Frame::new(Address::ZERO, Address::ZERO, 0, 0, vec![], 32);
            frame.pc = 1;
            let opcode = 0x5f + width as u8;
            push_immediate(&mut frame, &[opcode, 0xab], opcode).unwrap();
            let mut expected = [0; 32];
            expected[32 - width] = 0xab;
            assert_eq!(frame.stack, vec![Word::from_be_slice(&expected)]);
            assert_eq!(frame.pc, 1 + width);
        }
    }
}
