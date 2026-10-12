use crate::{
    error::VmError,
    memory::Memory,
    primitives::{Address, Word},
};

pub(super) const STACK_LIMIT: usize = 1024;

/// Ephemeral execution state for one contract invocation.
#[derive(Debug)]
pub(super) struct Frame {
    pub(super) pc: usize,
    pub(super) stack: Vec<Word>,
    pub(super) memory: Memory,
    pub(super) input: Vec<u8>,
    pub(super) caller: Address,
    pub(super) address: Address,
    pub(super) call_value: u64,
    pub(super) return_data: Vec<u8>,
}

impl Frame {
    pub(super) fn new(
        address: Address,
        caller: Address,
        call_value: u64,
        input: Vec<u8>,
        memory_limit: usize,
    ) -> Self {
        Self {
            pc: 0,
            stack: Vec::new(),
            memory: Memory::with_limit(memory_limit),
            input,
            caller,
            address,
            call_value,
            return_data: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, value: Word) -> Result<(), VmError> {
        if self.stack.len() >= STACK_LIMIT {
            return Err(VmError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    pub(super) fn pop(&mut self) -> Result<Word, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    pub(super) fn pop2(&mut self) -> Result<(Word, Word), VmError> {
        // Operand zero is the top of the stack (Yellow Paper convention).
        let first = self.pop()?;
        let second = self.pop()?;
        Ok((first, second))
    }

    pub(super) fn duplicate(&mut self, depth: usize) -> Result<(), VmError> {
        let index = self.stack.len().checked_sub(depth).ok_or(VmError::StackUnderflow)?;
        self.push(*self.stack.get(index).ok_or(VmError::StackUnderflow)?)
    }

    pub(super) fn swap(&mut self, depth: usize) -> Result<(), VmError> {
        let top = self.stack.len().checked_sub(1).ok_or(VmError::StackUnderflow)?;
        let other = top.checked_sub(depth).ok_or(VmError::StackUnderflow)?;
        self.stack.swap(top, other);
        Ok(())
    }
}
