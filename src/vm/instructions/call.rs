//! CALL validation, child execution, gas settlement, and rollback.

use super::super::{
    engine::{CallContext, Evm},
    frame::Frame,
};
use super::memory::{expand_memory, memory_range};
use crate::{
    error::VmError,
    gas::GasMeter,
    primitives::{Address, Word},
    trace::Tracer,
};

pub(super) fn call(
    evm: &mut Evm,
    frame: &mut Frame,
    gas: &mut GasMeter,
    tracer: &mut Option<&mut dyn Tracer>,
) -> Result<(), VmError> {
    // Top first: gas, to, value, input offset/size, output offset/size.
    let gas_limit = frame.pop()?.low_u64_bounded().unwrap_or(u64::MAX);
    let target = Address::from_word(frame.pop()?);
    let value_word = frame.pop()?;
    let (in_offset, in_size) = frame.pop2()?;
    let (out_offset, out_size) = frame.pop2()?;
    let (in_offset, in_size) = memory_range(in_offset, in_size)?;
    let (out_offset, out_size) = memory_range(out_offset, out_size)?;
    expand_memory(frame, gas, out_offset, out_size)?;
    expand_memory(frame, gas, in_offset, in_size)?;
    frame.return_data.clear();

    // A rejected CALL never starts a child and never spends forwarded gas.
    let Some(value) = value_word.low_u64_bounded() else {
        return frame.push(Word::ZERO);
    };
    let Some(depth) = frame.depth.checked_add(1).filter(|depth| *depth < evm.config.max_call_depth)
    else {
        return frame.push(Word::ZERO);
    };
    if evm.state.balance_of(frame.address) < value {
        return frame.push(Word::ZERO);
    }
    let target_balance = if target == frame.address {
        None
    } else {
        let Some(balance) = evm.state.balance_of(target).checked_add(value) else {
            // Local rule for this project's u64 balance domain.
            return frame.push(Word::ZERO);
        };
        Some(balance)
    };

    let input = frame.memory.read(in_offset, in_size)?;
    let forwarded_gas = gas.forward(gas_limit)?;
    let checkpoint = evm.state.snapshot();
    if let Some(balance) = target_balance {
        evm.state.account_mut(frame.address).balance -= value;
        evm.state.account_mut(target).balance = balance;
    }

    match evm.execute(
        CallContext { address: target, caller: frame.address, value, depth },
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
