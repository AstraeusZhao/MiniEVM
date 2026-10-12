use std::fmt;

/// A fixed 256-bit EVM word. Arithmetic wraps modulo 2^256.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Word(pub [u64; 4]);

impl Word {
    pub const ZERO: Self = Self([0; 4]);
    pub const ONE: Self = Self([1, 0, 0, 0]);
    pub const fn from_u64(value: u64) -> Self {
        Self([value, 0, 0, 0])
    }
    pub fn low_u64(self) -> u64 {
        self.0[0]
    }
    /// Returns the low u64 limb only when the upper 192 bits are zero.
    /// Used where a word addresses a 64-bit world (memory, calldata, jumps)
    /// so that large values are treated as out of range instead of silently
    /// truncating into a valid small offset.
    pub fn low_u64_bounded(self) -> Option<u64> {
        if self.0[1] | self.0[2] | self.0[3] == 0 { Some(self.0[0]) } else { None }
    }
    pub fn is_zero(self) -> bool {
        self.0 == [0; 4]
    }
    pub fn wrapping_add(self, rhs: Self) -> Self {
        let (a, c0) = self.0[0].overflowing_add(rhs.0[0]);
        let (b0, c1) = self.0[1].overflowing_add(rhs.0[1]);
        let (b, c2) = b0.overflowing_add(c0 as u64);
        let (c0v, c3) = self.0[2].overflowing_add(rhs.0[2]);
        let (c, c4) = c0v.overflowing_add((c1 || c2) as u64);
        let (d0, _) = self.0[3].overflowing_add(rhs.0[3]);
        let (d, _) = d0.overflowing_add((c3 || c4) as u64);
        Self([a, b, c, d])
    }
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        let (a, b0) = self.0[0].overflowing_sub(rhs.0[0]);
        let (b1, b1o) = self.0[1].overflowing_sub(rhs.0[1]);
        let (b, b2) = b1.overflowing_sub(b0 as u64);
        let (c1, b3) = self.0[2].overflowing_sub(rhs.0[2]);
        let (c, b4) = c1.overflowing_sub((b1o || b2) as u64);
        let (d1, _) = self.0[3].overflowing_sub(rhs.0[3]);
        let (d, _) = d1.overflowing_sub((b3 || b4) as u64);
        Self([a, b, c, d])
    }
    pub fn and(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] & rhs.0[i]))
    }
    pub fn or(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] | rhs.0[i]))
    }
    pub fn xor(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] ^ rhs.0[i]))
    }
    pub fn invert(self) -> Self {
        Self(std::array::from_fn(|i| !self.0[i]))
    }
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        let mut result = Self::ZERO;
        for i in 0..4 {
            for j in 0..(4 - i) {
                let (low, high) = multiply(self.0[i], rhs.0[j]);
                add_limb(&mut result.0, i + j, low);
                if i + j + 1 < 4 {
                    add_limb(&mut result.0, i + j + 1, high);
                }
            }
        }
        result
    }
    pub fn unsigned_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }

        let mut quotient = Self::ZERO;
        let mut remainder = Self::ZERO;
        for bit_index in (0..256).rev() {
            let overflow = remainder.0[3] >> 63 != 0;
            remainder = remainder.shl1();
            if self.bit(bit_index) {
                remainder.0[0] |= 1;
            }
            if overflow || remainder >= rhs {
                remainder = remainder.wrapping_sub(rhs);
                quotient.set_bit(bit_index);
            }
        }
        quotient
    }
    /// Returns the EVM BYTE result for a byte index in the range 0..=31.
    pub fn byte(self, index: u64) -> Self {
        if index >= 32 {
            return Self::ZERO;
        }
        let bytes = self.to_be_bytes();
        Self::from_u64(bytes[index as usize] as u64)
    }
    pub fn from_be_slice(bytes: &[u8]) -> Self {
        let mut result = [0u64; 4];
        for (i, byte) in bytes.iter().rev().take(32).enumerate() {
            result[i / 8] |= (*byte as u64) << ((i % 8) * 8);
        }
        Self(result)
    }
    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for (index, part) in self.0.iter().rev().enumerate() {
            bytes[index * 8..(index + 1) * 8].copy_from_slice(&part.to_be_bytes());
        }
        bytes
    }

    fn bit(self, index: usize) -> bool {
        (self.0[index / 64] & (1u64 << (index % 64))) != 0
    }

    fn set_bit(&mut self, index: usize) {
        self.0[index / 64] |= 1u64 << (index % 64);
    }

    fn shl1(self) -> Self {
        let mut result = [0u64; 4];
        let mut carry = 0;
        for (index, limb) in self.0.iter().copied().enumerate() {
            result[index] = (limb << 1) | carry;
            carry = limb >> 63;
        }
        Self(result)
    }
}
fn multiply(left: u64, right: u64) -> (u64, u64) {
    let value = (left as u128) * (right as u128);
    (value as u64, (value >> 64) as u64)
}
fn add_limb(limbs: &mut [u64; 4], index: usize, value: u64) {
    let (sum, mut carry) = limbs[index].overflowing_add(value);
    limbs[index] = sum;
    let mut cursor = index + 1;
    while carry && cursor < limbs.len() {
        (limbs[cursor], carry) = limbs[cursor].overflowing_add(1);
        cursor += 1;
    }
}
impl fmt::Debug for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "0x")?;
        for part in self.0.iter().rev() {
            write!(formatter, "{part:016x}")?;
        }
        Ok(())
    }
}

impl Ord for Word {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}
impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
