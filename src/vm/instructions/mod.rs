//! Opcode dispatch and instruction semantics.

use crate::{
    error::VmError,
    gas::GasMeter,
    primitives::{Address, Word},
    trace::Tracer,
};

mod call;
mod log;
mod memory;
mod stack;

use super::{bytecode::AnalyzedCode, calldata, engine::Evm, frame::Frame};
use call::call;
use log::emit_log;
use memory::{
    copy_calldata_to_memory, copy_return_data, copy_to_memory, expand_memory, memory_index,
    memory_range,
};
use stack::push_immediate;

/// Executes one opcode. `Some` signals a normal frame return.
pub(super) fn dispatch(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut GasMeter,
    code: &AnalyzedCode,
    opcode: u8,
    tracer: &mut Option<&mut dyn Tracer>,
) -> Result<Option<Vec<u8>>, VmError> {
    match opcode {
        0x00 => return Ok(Some(vec![])),
        0x01 => binary(frame, Word::wrapping_add)?,
        0x02 => binary(frame, Word::wrapping_mul)?,
        0x03 => binary(frame, Word::wrapping_sub)?,
        0x04 => binary(frame, Word::unsigned_div)?,
        0x06 => binary(frame, Word::unsigned_mod)?,
        0x10 => compare(frame, |left, right| left < right)?,
        0x11 => compare(frame, |left, right| left > right)?,
        0x14 => compare(frame, |left, right| left == right)?,
        0x15 => {
            let value = frame.pop()?;
            frame.push(Word::from_u64(value.is_zero() as u64))?;
        }
        0x16 => binary(frame, Word::and)?,
        0x17 => binary(frame, Word::or)?,
        0x18 => binary(frame, Word::xor)?,
        0x19 => {
            let value = frame.pop()?;
            frame.push(value.invert())?;
        }
        0x1a => {
            let index = frame.pop()?.low_u64_bounded().unwrap_or(u64::MAX);
            let value = frame.pop()?;
            frame.push(value.byte(index))?;
        }
        0x1b..=0x1d => {
            let (shift, value) = frame.pop2()?;
            let shifted = match opcode {
                0x1b => value.shift_left(shift),
                0x1c => value.shift_right(shift),
                _ => value.shift_arithmetic_right(shift),
            };
            frame.push(shifted)?;
        }
        0x30 => frame.push(Word::from_be_slice(&frame.address.0))?,
        0x31 => {
            let account = Address::from_word(frame.pop()?);
            frame.push(Word::from_u64(evm.state.balance_of(account)))?;
        }
        0x33 => frame.push(Word::from_be_slice(&frame.caller.0))?,
        0x34 => frame.push(Word::from_u64(frame.call_value))?,
        0x35 => {
            // Offsets beyond the 64-bit addressable range read zero, as in
            // the EVM, instead of silently truncating to a small offset.
            let offset = match frame
                .pop()?
                .low_u64_bounded()
                .and_then(|value| usize::try_from(value).ok())
            {
                Some(offset) => offset,
                None => {
                    frame.push(Word::ZERO)?;
                    return Ok(None);
                }
            };
            frame.push(calldata::load_word(&frame.input, offset))?;
        }
        0x36 => frame.push(Word::from_u64(frame.input.len() as u64))?,
        0x37 => copy_calldata_to_memory(frame, gas)?,
        0x38 => frame.push(Word::from_u64(code.bytes().len() as u64))?,
        0x39 => copy_to_memory(frame, gas, code.bytes())?,
        0x3d => frame.push(Word::from_u64(frame.return_data.len() as u64))?,
        0x3e => copy_return_data(frame, gas)?,
        0x41 => frame.push(Word::from_be_slice(&evm.block.coinbase.0))?,
        0x42 => frame.push(Word::from_u64(evm.block.timestamp))?,
        0x43 => frame.push(Word::from_u64(evm.block.number))?,
        0x46 => frame.push(Word::from_u64(evm.block.chain_id))?,
        0x50 => {
            frame.pop()?;
        }
        0x51 => {
            let offset = memory_index(frame.pop()?)?;
            expand_memory(frame, gas, offset, 32)?;
            let bytes = frame.memory.read(offset, 32)?;
            frame.push(Word::from_be_slice(&bytes))?;
        }
        0x52 => {
            let (offset, value) = frame.pop2()?;
            let offset = memory_index(offset)?;
            expand_memory(frame, gas, offset, 32)?;
            frame.memory.write(offset, &value.to_be_bytes())?;
        }
        0x53 => {
            let (offset, value) = frame.pop2()?;
            let offset = memory_index(offset)?;
            expand_memory(frame, gas, offset, 1)?;
            frame.memory.write(offset, &[value.low_u64() as u8])?;
        }
        0x54 => {
            let key = frame.pop()?;
            let value = evm
                .state
                .account(frame.address)
                .and_then(|account| account.storage.get(&key))
                .copied()
                .unwrap_or(Word::ZERO);
            frame.push(value)?;
        }
        0x55 => {
            let (key, value) = frame.pop2()?;
            let storage = &mut evm.state.account_mut(frame.address).storage;
            if value.is_zero() {
                storage.remove(&key);
            } else {
                storage.insert(key, value);
            }
        }
        0x56 => {
            let target = frame.pop()?;
            let target = target
                .low_u64_bounded()
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or(usize::MAX);
            jump(frame, code, target)?;
        }
        0x57 => {
            let (target, condition) = frame.pop2()?;
            if !condition.is_zero() {
                let target = target
                    .low_u64_bounded()
                    .and_then(|value| usize::try_from(value).ok())
                    .unwrap_or(usize::MAX);
                jump(frame, code, target)?;
            }
        }
        0x58 => frame.push(Word::from_u64(frame.pc.saturating_sub(1) as u64))?,
        0x59 => frame.push(Word::from_u64(frame.memory.len() as u64))?,
        0x5a => frame.push(Word::from_u64(gas.remaining()))?,
        0x5b => {}
        0x5f => frame.push(Word::ZERO)?,
        0x60..=0x7f => push_immediate(frame, code.bytes(), opcode)?,
        0x80..=0x8f => frame.duplicate((opcode - 0x7f) as usize)?,
        0x90..=0x9f => frame.swap((opcode - 0x8f) as usize)?,
        0xa0..=0xa4 => emit_log(evm, frame, gas, (opcode - 0xa0) as usize)?,
        0xf1 => call(evm, frame, gas, tracer)?,
        0xf3 => {
            let (offset, size) = frame.pop2()?;
            let (offset, size) = memory_range(offset, size)?;
            expand_memory(frame, gas, offset, size)?;
            return Ok(Some(frame.memory.read(offset, size)?));
        }
        0xfd => {
            let (offset, size) = frame.pop2()?;
            let (offset, size) = memory_range(offset, size)?;
            expand_memory(frame, gas, offset, size)?;
            return Err(VmError::RevertedWithGas {
                output: frame.memory.read(offset, size)?,
                remaining: gas.remaining(),
            });
        }
        _ => return Err(VmError::InvalidOpcode(opcode)),
    }
    Ok(None)
}

fn binary(frame: &mut Frame, operation: fn(Word, Word) -> Word) -> Result<(), VmError> {
    let (left, right) = frame.pop2()?;
    frame.push(operation(left, right))
}
fn compare(frame: &mut Frame, predicate: fn(Word, Word) -> bool) -> Result<(), VmError> {
    let (left, right) = frame.pop2()?;
    frame.push(Word::from_u64(predicate(left, right) as u64))
}
fn jump(frame: &mut Frame, code: &AnalyzedCode, target: usize) -> Result<(), VmError> {
    if code.is_valid_jumpdest(target) {
        frame.pc = target;
        Ok(())
    } else {
        Err(VmError::InvalidJump(target))
    }
}
