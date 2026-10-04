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