//! Log operands and dynamic charges.

use super::super::{engine::Evm, frame::Frame};
use super::memory::{expand_memory, memory_range};
use crate::{error::VmError, gas::GasMeter, state::LogEntry};

pub(super) fn emit_log(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut GasMeter,
    topic_count: usize,
) -> Result<(), VmError> {
    // Pop memory offset and size first, then topic0..topicN.
    let (offset, size) = frame.pop2()?;
    let (offset, size) = memory_range(offset, size)?;
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
