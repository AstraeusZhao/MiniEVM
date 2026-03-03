use crate::error::VmError;

/// Frame-local linear memory with an explicit expansion limit.
#[derive(Debug, Clone, Default)]
pub struct Memory {
    bytes: Vec<u8>,
    max_bytes: usize,
}

impl Memory {
    pub fn with_limit(max_bytes: usize) -> Self {
        Self { bytes: Vec::new(), max_bytes }
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn read(&mut self, offset: usize, size: usize) -> Result<Vec<u8>, VmError> {
        if size == 0 {
            return Ok(Vec::new());
        }
        let end = offset.checked_add(size).ok_or(VmError::MemoryLimitExceeded)?;
        self.expand(end)?;
        Ok(self.bytes[offset..end].to_vec())
    }
    pub fn write(&mut self, offset: usize, value: &[u8]) -> Result<(), VmError> {
        let end = offset.checked_add(value.len()).ok_or(VmError::MemoryLimitExceeded)?;
        self.expand(end)?;
        self.bytes[offset..end].copy_from_slice(value);
        Ok(())
    }
    fn expand(&mut self, end: usize) -> Result<(), VmError> {
        if end > self.max_bytes {
            return Err(VmError::MemoryLimitExceeded);
        }
        if self.bytes.len() < end {
            self.bytes.resize(end, 0);
        }
        Ok(())
    }
}
