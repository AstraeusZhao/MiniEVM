use crate::error::VmError;

/// Tracks gas available to one execution frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GasMeter {
    limit: u64,
    remaining: u64,
}

impl GasMeter {
    pub const fn new(limit: u64) -> Self {
        Self { limit, remaining: limit }
    }
    pub const fn remaining(self) -> u64 {
        self.remaining
    }
    pub const fn used(self) -> u64 {
        self.limit - self.remaining
    }
    pub fn charge(&mut self, amount: u64) -> Result<(), VmError> {
        self.remaining = self.remaining.checked_sub(amount).ok_or(VmError::OutOfGas)?;
        Ok(())
    }

    /// Caps the forwarded gas to what this frame still has, using the EIP-150 63/64 cap.
    /// This simplified model does not yet implement the value stipend or fork-specific costs.
    pub fn forward(&mut self, requested: u64) -> Result<u64, VmError> {
        let actual = requested.min(self.remaining - self.remaining / 64);
        self.charge(actual)?;
        Ok(actual)
    }

    /// Returns the child frame's unused gas to this frame after a sub-call.
    pub fn settle_child(&mut self, forwarded: u64, child_remaining: u64) {
        self.remaining =
            self.remaining.saturating_add(child_remaining.min(forwarded)).min(self.limit);
    }
}
