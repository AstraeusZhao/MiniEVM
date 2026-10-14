//! Checked memory operands, expansion, and copying.

use super::super::frame::Frame;
use crate::{error::VmError, gas::GasMeter, primitives::Word};

/// Copies CALLDATA/CODE into memory. Stack layout: [mem_offset, src_offset, size].
pub(super) fn copy_to_memory(
    frame: &mut Frame,
    gas: &mut GasMeter,
    source: &[u8],
) -> Result<(), VmError> {
    let (memory_offset, source_offset, size) = copy_operands(frame)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    frame.memory.copy_padded_from(memory_offset, size, source, source_offset)
}

pub(super) fn copy_calldata_to_memory(
    frame: &mut Frame,
    gas: &mut GasMeter,
) -> Result<(), VmError> {
    let (memory_offset, source_offset, size) = copy_operands(frame)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    frame.memory.copy_padded_from(memory_offset, size, &frame.input, source_offset)
}

fn copy_operands(frame: &mut Frame) -> Result<(usize, usize, usize), VmError> {
    let memory_offset = frame.pop()?;
    let source_offset = frame
        .pop()?
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    let (memory_offset, size) = memory_range(memory_offset, frame.pop()?)?;
    Ok((memory_offset, source_offset, size))
}

/// Zero-length ranges do not access memory, so their offsets need not fit usize.
pub(super) fn memory_range(offset: Word, size: Word) -> Result<(usize, usize), VmError> {
    let size = memory_index(size)?;
    let offset = if size == 0 { 0 } else { memory_index(offset)? };
    Ok((offset, size))
}

pub(super) fn memory_index(value: Word) -> Result<usize, VmError> {
    value
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(VmError::MemoryLimitExceeded)
}

/// Copies RETURNDATA into memory. Stack layout: [mem_offset, data_offset, size].
pub(super) fn copy_return_data(frame: &mut Frame, gas: &mut GasMeter) -> Result<(), VmError> {
    let memory_offset = frame.pop()?;
    let data_offset = frame
        .pop()?
        .low_u64_bounded()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(VmError::ReturnDataOutOfBounds)?;
    let (memory_offset, size) = memory_range(memory_offset, frame.pop()?)?;
    expand_memory(frame, gas, memory_offset, size)?;
    charge_copy(gas, size)?;
    let end = data_offset.checked_add(size).ok_or(VmError::ReturnDataOutOfBounds)?;
    let bytes = frame.return_data.get(data_offset..end).ok_or(VmError::ReturnDataOutOfBounds)?;
    frame.memory.write(memory_offset, bytes)
}

pub(super) fn expand_memory(
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
