use std::fmt;

/// A 20-byte account identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const ZERO: Self = Self([0; 20]);
    pub const fn from_last_byte(value: u8) -> Self {
        let mut bytes = [0; 20];
        bytes[19] = value;
        Self(bytes)
    }

    pub fn from_word(value: crate::primitives::Word) -> Self {
        let bytes = value.to_be_bytes();
        let mut address = [0u8; 20];
        address.copy_from_slice(&bytes[12..]);
        Self(address)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "0x")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
