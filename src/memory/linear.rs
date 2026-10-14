use crate::error::VmError;

/// Frame-local linear memory with an explicit expansion limit.
/// Expansion is rounded up to 32 bytes. `Default` permits no nonempty ranges;
/// use `with_limit` to choose a usable capacity.
#[derive(Debug, Clone, Default)]
pub struct Memory {
    bytes: Vec<u8>,
    max_bytes: usize,
}

impl Memory {
    /// Creates empty memory with a maximum allocated length in bytes.
    pub fn with_limit(max_bytes: usize) -> Self {
        Self { bytes: Vec::new(), max_bytes }
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn expansion_words(&self, offset: usize, size: usize) -> Result<u64, VmError> {
        let new_len = self.required_len(offset, size)?;
        u64::try_from(new_len / 32).map_err(|_| VmError::MemoryLimitExceeded)
    }
    pub fn ensure_range(&mut self, offset: usize, size: usize) -> Result<(), VmError> {
        if size == 0 {
            return Ok(());
        }
        self.expand(self.required_len(offset, size)?)
    }
    pub fn read(&mut self, offset: usize, size: usize) -> Result<Vec<u8>, VmError> {
        if size == 0 {
            return Ok(Vec::new());
        }
        let end = offset.checked_add(size).ok_or(VmError::MemoryLimitExceeded)?;
        self.expand(self.required_len(offset, size)?)?;
        let mut output = Vec::new();
        output.try_reserve_exact(size).map_err(|_| VmError::MemoryLimitExceeded)?;
        output.extend_from_slice(&self.bytes[offset..end]);
        Ok(output)
    }
    pub fn write(&mut self, offset: usize, value: &[u8]) -> Result<(), VmError> {
        if value.is_empty() {
            return Ok(());
        }
        let end = offset.checked_add(value.len()).ok_or(VmError::MemoryLimitExceeded)?;
        self.expand(self.required_len(offset, value.len())?)?;
        self.bytes[offset..end].copy_from_slice(value);
        Ok(())
    }

    /// Copies a source range into memory and zero-fills bytes beyond the source.
    pub fn copy_padded_from(
        &mut self,
        offset: usize,
        size: usize,
        source: &[u8],
        source_offset: usize,
    ) -> Result<(), VmError> {
        self.ensure_range(offset, size)?;
        if size == 0 {
            return Ok(());
        }

        let destination = &mut self.bytes[offset..offset + size];
        destination.fill(0);
        if let Some(source) = source.get(source_offset..) {
            let copy_len = source.len().min(size);
            destination[..copy_len].copy_from_slice(&source[..copy_len]);
        }
        Ok(())
    }

    fn expand(&mut self, end: usize) -> Result<(), VmError> {
        if end > self.max_bytes {
            return Err(VmError::MemoryLimitExceeded);
        }
        if self.bytes.len() < end {
            self.bytes
                .try_reserve(end - self.bytes.len())
                .map_err(|_| VmError::MemoryLimitExceeded)?;
            self.bytes.resize(end, 0);
        }
        Ok(())
    }

    fn required_len(&self, offset: usize, size: usize) -> Result<usize, VmError> {
        if size == 0 {
            return Ok(self.bytes.len());
        }
        let end = offset.checked_add(size).ok_or(VmError::MemoryLimitExceeded)?;
        let aligned_end = end.div_ceil(32).checked_mul(32).ok_or(VmError::MemoryLimitExceeded)?;
        if aligned_end > self.max_bytes {
            return Err(VmError::MemoryLimitExceeded);
        }
        Ok(aligned_end.max(self.bytes.len()))
    }
}
