//! Opcode dispatch and instruction semantics.

use crate::{
    error::VmError,
    gas::GasMeter,
    primitives::{Address, Word},
    state::LogEntry,
};

use super::{
    bytecode::AnalyzedCode,
    calldata,
    engine::{CallContext, Evm},
    frame::Frame,
};
use crate::trace::Tracer;

/// Executes one opcode. `Some` signals a normal frame return.
pub(super) fn dispatch(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut crate::gas::GasMeter,
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
        0x10 => compare(frame, |left, right| left < right)?,
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
        0x42 => frame.push(Word::from_u64(evm.block.timestamp))?,
        0x43 => frame.push(Word::from_u64(evm.block.number))?,
        0x41 => frame.push(Word::from_be_slice(&evm.block.coinbase.0))?,
        0x46 => frame.push(Word::from_u64(evm.block.chain_id))?,
        0x58 => frame.push(Word::from_u64(frame.pc.saturating_sub(1) as u64))?,
        0x59 => frame.push(Word::from_u64(frame.memory.len() as u64))?,
        0x5a => frame.push(Word::from_u64(gas.remaining()))?,
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
        0x5b => {}
        0x60..=0x7f => push_immediate(frame, code.bytes(), opcode)?,
        0x80..=0x8f => frame.duplicate((opcode - 0x7f) as usize)?,
        0x90..=0x9f => frame.swap((opcode - 0x8f) as usize)?,
        0xa0..=0xa4 => emit_log(evm, frame, gas, (opcode - 0xa0) as usize)?,
        0xf1 => call(evm, frame, gas, tracer)?,
        0xf3 => {
            let (offset, size) = frame.pop2()?;
            let offset = memory_index(offset)?;
            let size = memory_index(size)?;
            expand_memory(frame, gas, offset, size)?;
            return Ok(Some(frame.memory.read(offset, size)?));
        }
        0xfd => {
            let (offset, size) = frame.pop2()?;
            let offset = memory_index(offset)?;
            let size = memory_index(size)?;
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
fn push_immediate(frame: &mut Frame, code: &[u8], opcode: u8) -> Result<(), VmError> {
    let width = (opcode - 0x5f) as usize;
    let end = frame.pc.checked_add(width).ok_or(VmError::CodeOutOfBounds)?;
    let mut bytes = vec![0; width];
    let available = code.len().saturating_sub(frame.pc).min(width);
    bytes[..available].copy_from_slice(&code[frame.pc..frame.pc + available]);
    frame.push(Word::from_be_slice(&bytes))?;
    frame.pc = end;
    Ok(())
}
fn emit_log(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut GasMeter,
    topic_count: usize,
) -> Result<(), VmError> {
    // Pop memory offset and size first, then topic0..topicN.
    let offset = memory_index(frame.pop()?)?;
    let size = memory_index(frame.pop()?)?;
    let mut topics = Vec::with_capacity(topic_count);
    for _ in 0..topic_count {
        topics.push(frame.pop()?);
    }
    expand_memory(frame, gas, offset, size)?;
    let dynamic_cost = (topic_count as u128) * 375 + (size as u128) * 8;
    gas.charge(u64::try_from(dynamic_cost).map_err(|_| VmError::OutOfGas)?)?;
    let data = frame.memory.read(offset, size)?;
    evm.state.emit_log(LogEntry { address: frame.address, topics, data });
    Ok(())
}

/// Copies CALLDATA/CODE into memory. Stack layout: [mem_offset, src_offset, size].
fn copy_to_memory(frame: &mut Frame, gas: &mut GasMeter, source: &[u8]) -> Result<(), VmError> {
    let (memory_offset, source_offset, size) = copy_operands(frame)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    frame.memory.copy_padded_from(memory_offset, size, source, source_offset)
}

fn copy_calldata_to_memory(frame: &mut Frame, gas: &mut GasMeter) -> Result<(), VmError> {
    let (memory_offset, source_offset, size) = copy_operands(frame)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    frame.memory.copy_padded_from(memory_offset, size, &frame.input, source_offset)
}

fn copy_operands(frame: &mut Frame) -> Result<(usize, usize, usize), VmError> {
    let memory_offset = memory_index(frame.pop()?)?;
    let source_offset = frame
        .pop()?
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    let size = memory_index(frame.pop()?)?;
    Ok((memory_offset, source_offset, size))
}

fn memory_index(value: Word) -> Result<usize, VmError> {
    value
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(VmError::MemoryLimitExceeded)
}

/// Copies RETURNDATA into memory. Stack layout: [mem_offset, data_offset, size].
fn copy_return_data(frame: &mut Frame, gas: &mut GasMeter) -> Result<(), VmError> {
    let memory_offset = memory_index(frame.pop()?)?;
    let data_offset = frame
        .pop()?
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(VmError::ReturnDataOutOfBounds)?;
    let size = memory_index(frame.pop()?)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    let end = data_offset.checked_add(size).ok_or(VmError::ReturnDataOutOfBounds)?;
    let bytes = frame.return_data.get(data_offset..end).ok_or(VmError::ReturnDataOutOfBounds)?;
    frame.memory.write(memory_offset, bytes)
}

fn call(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut GasMeter,
    tracer: &mut Option<&mut dyn Tracer>,
) -> Result<(), VmError> {
    // Top first: gas, to, value, input offset/size, output offset/size.
    let gas_limit = frame.pop()?.low_u64_bounded().unwrap_or(u64::MAX);
    let target = Address::from_word(frame.pop()?);
    let value_word = frame.pop()?;
    let in_offset = memory_index(frame.pop()?)?;
    let in_size = memory_index(frame.pop()?)?;
    let out_offset = memory_index(frame.pop()?)?;
    let out_size = memory_index(frame.pop()?)?;
    expand_memory(frame, gas, out_offset, out_size)?;
    expand_memory(frame, gas, in_offset, in_size)?;

    let input = frame.memory.read(in_offset, in_size)?;
    let forwarded_gas = gas.forward(gas_limit)?;
    let checkpoint = evm.state.snapshot();

    // A value above the 64-bit balance domain can never be paid. No
    // sub-call runs, so the forwarded gas must be returned to this frame.
    let value = match value_word.low_u64_bounded() {
        Some(value) => value,
        None => {
            frame.return_data.clear();
            gas.settle_child(forwarded_gas, forwarded_gas);
            return frame.push(Word::ZERO);
        }
    };
    if evm.state.balance_of(frame.address) < value {
        // Same as above: no sub-call executes, refund all forwarded gas.
        frame.return_data.clear();
        gas.settle_child(forwarded_gas, forwarded_gas);
        return frame.push(Word::ZERO);
    }
    evm.state.account_mut(frame.address).balance -= value;
    evm.state.account_mut(target).balance =
        evm.state.balance_of(target).checked_add(value).ok_or(VmError::ValueOverflow)?;

    match evm.execute(
        CallContext { address: target, caller: frame.address, value, depth: frame.depth + 1 },
        input,
        forwarded_gas,
        tracer,
    ) {
        Ok((output, child_remaining)) => {
            gas.settle_child(forwarded_gas, child_remaining);
            frame.return_data = output;
            write_return_data(frame, out_offset, out_size)?;
            frame.push(Word::ONE)
        }
        Err(VmError::RevertedWithGas { output, remaining }) => {
            evm.state.restore(checkpoint);
            gas.settle_child(forwarded_gas, remaining);
            frame.return_data = output;
            write_return_data(frame, out_offset, out_size)?;
            frame.push(Word::ZERO)
        }
        Err(_) => {
            evm.state.restore(checkpoint);
            frame.return_data.clear();
            frame.push(Word::ZERO)
        }
    }
}

/// Extend the requested return area, but overwrite only actual returned bytes.
fn write_return_data(frame: &mut Frame, out_offset: usize, out_size: usize) -> Result<(), VmError> {
    frame.memory.ensure_range(out_offset, out_size)?;
    let copy_len = out_size.min(frame.return_data.len());
    frame.memory.write(out_offset, &frame.return_data[..copy_len])
}

fn expand_memory(
    frame: &mut Frame,
    gas: &mut GasMeter,
    offset: usize,
    size: usize,
) -> Result<(), VmError> {
    let current_words = u64::try_from(frame.memory.len() / 32).map_err(|_| VmError::OutOfGas)?;
    let new_words = frame.memory.expansion_words(offset, size)?;
    let cost = memory_cost(new_words)?
        .checked_sub(memory_cost(current_words)?)
        .ok_or(VmError::OutOfGas)?;
    gas.charge(cost)?;
    frame.memory.ensure_range(offset, size)
}

fn memory_cost(words: u64) -> Result<u64, VmError> {
    let words = words as u128;
    let cost = words * 3 + words * words / 512;
    u64::try_from(cost).map_err(|_| VmError::OutOfGas)
}

fn charge_copy(gas: &mut GasMeter, size: usize) -> Result<(), VmError> {
    let words = u64::try_from(size.div_ceil(32)).map_err(|_| VmError::OutOfGas)?;
    gas.charge(words.checked_mul(3).ok_or(VmError::OutOfGas)?)
}
