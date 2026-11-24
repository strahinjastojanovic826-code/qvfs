use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quat {
    Q0 = 0b00, // A
    Q1 = 0b01, // C
    Q2 = 0b10, // G
    Q3 = 0b11, // T
}

impl Quat {
    #[inline]
    pub fn from_u8_masked(val: u8) -> Self {
        match val & 0b11 {
            0b00 => Quat::Q0,
            0b01 => Quat::Q1,
            0b10 => Quat::Q2,
            0b11 => Quat::Q3,
            // SAFETY: `val & 0b11` masks all bits except the lowest two, so the result
            // is always in `0..=3` and matches `0b00..=0b11` (`Quat::Q0..=Quat::Q3`).
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn as_char(self) -> char {
        match self {
            Quat::Q0 => 'A',
            Quat::Q1 => 'C',
            Quat::Q2 => 'G',
            Quat::Q3 => 'T',
        }
    }
    pub fn q_not(self) -> Self {
        match self {
            Quat::Q0 => Quat::Q3,
            Quat::Q1 => Quat::Q2,
            Quat::Q2 => Quat::Q1,
            Quat::Q3 => Quat::Q0,
        }
    }

    /// Quaternary AND operation (Bitwise minimum / intersection)
    pub fn q_and(self, other: Self) -> Self {
        let val = (self as u8) & (other as u8);
        Quat::from_u8_masked(val)
    }

    /// Quaternary OR operation (Bitwise maximum / union)
    pub fn q_or(self, other: Self) -> Self {
        let val = (self as u8) | (other as u8);
        Quat::from_u8_masked(val)
    }

    /// Quaternary XOR operation (Difference / modulo-4 addition logic)
    pub fn q_xor(self, other: Self) -> Self {
        let val = ((self as u8) ^ (other as u8)) & 0b11;
        Quat::from_u8_masked(val)
    }
    /// Biological DNA complement: Q0(A) <-> Q3(T), Q1(C) <-> Q2(G).
    /// Note: Since Q0=0b00, Q1=0b01, Q2=0b10, Q3=0b11, complement is bitwise NOT masked to 2 bits.
    #[inline]
    pub fn complement(self) -> Self {
        self.q_not()
    }

    /// Modulo-4 addition: (self + other) mod 4.
    /// Useful for phase shifts, cyclic state machines, and quaternary checksums.
    #[inline]
    pub fn q_add_mod4(self, other: Self) -> Self {
        let val = ((self as u8).wrapping_add(other as u8)) & 0b11;
        Self::from_u8_masked(val)
    }

    /// Modulo-4 subtraction: (self - other) mod 4.
    #[inline]
    pub fn q_sub_mod4(self, other: Self) -> Self {
        let val = ((self as u8).wrapping_sub(other as u8)) & 0b11;
        Self::from_u8_masked(val)
    }

    /// Cyclic shift right by 1 state: Q0->Q1->Q2->Q3->Q0.
    #[inline]
    pub fn q_inc(self) -> Self {
        let val = ((self as u8) + 1) & 0b11;
        Self::from_u8_masked(val)
    }

    /// Cyclic shift left by 1 state: Q0->Q3->Q2->Q1->Q0.
    #[inline]
    pub fn q_dec(self) -> Self {
        let val = (self as u8).wrapping_sub(1) & 0b11;
        Self::from_u8_masked(val)
    }

    /// Packs 4 Quats into a single byte (MSB-first layout matching QuatBuffer).
    #[inline]
    pub fn pack_4(q: [Quat; 4]) -> u8 {
        ((q[0] as u8) << 6) | ((q[1] as u8) << 4) | ((q[2] as u8) << 2) | (q[3] as u8)
    }

    /// Unpacks a single byte into 4 Quats (MSB-first layout matching QuatBuffer).
    #[inline]
    pub fn unpack_4(byte: u8) -> [Quat; 4] {
        [
            Self::from_u8_masked((byte >> 6) & 0b11),
            Self::from_u8_masked((byte >> 4) & 0b11),
            Self::from_u8_masked((byte >> 2) & 0b11),
            Self::from_u8_masked(byte & 0b11),
        ]
    }
}

impl TryFrom<u8> for Quat {
    type Error = crate::error::QuatError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Quat::Q0),
            0b01 => Ok(Quat::Q1),
            0b10 => Ok(Quat::Q2),
            0b11 => Ok(Quat::Q3),
            _ => Err(crate::error::QuatError::PermissionDenied(format!(
                "Invalid byte for Quat: {value:#b}"
            ))),
        }
    }
}

impl TryFrom<char> for Quat {
    type Error = crate::error::QuatError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_uppercase() {
            'A' => Ok(Quat::Q0),
            'C' => Ok(Quat::Q1),
            'G' => Ok(Quat::Q2),
            'T' => Ok(Quat::Q3),
            _ => Err(crate::error::QuatError::PermissionDenied(format!(
                "Invalid character for Quat: '{c}'"
            ))),
        }
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}