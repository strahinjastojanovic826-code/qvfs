use crate::quat::Quat;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuatBuffer {
    pub data: Vec<u8>,
    pub len_quats: usize,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(quat_capacity: usize) -> Self {
        let byte_capacity = (quat_capacity + 3) / 4;
        Self {
            data: Vec::with_capacity(byte_capacity),
            len_quats: 0,
        }
    }

    pub fn push(&mut self, quat: Quat) {
        let byte_index = self.len_quats / 4;
        let bit_shift = 6 - ((self.len_quats % 4) * 2);

        if byte_index >= self.data.len() {
            self.data.push(0);
        }

        self.data[byte_index] |= (quat as u8) << bit_shift;
        self.len_quats += 1;
    }

    pub fn push_slice(&mut self, quats: &[Quat]) {
        self.data.reserve((quats.len() + 3) / 4);
        for &quat in quats {
            self.push(quat);
        }
    }

    pub fn get(&self, index: usize) -> Option<Quat> {
        if index >= self.len_quats {
            return None;
        }

        let byte_index = index / 4;
        let bit_shift = 6 - ((index % 4) * 2);
        let val = (self.data[byte_index] >> bit_shift) & 0b11;

        Some(Quat::from_u8_masked(val))
    }

    pub fn set(&mut self, index: usize, quat: Quat) -> bool {
        if index >= self.len_quats {
            return false;
        }

        let byte_index = index / 4;
        let bit_shift = 6 - ((index % 4) * 2);

        let mask = !(0b11_u8 << bit_shift);
        self.data[byte_index] = (self.data[byte_index] & mask) | ((quat as u8) << bit_shift);
        true
    }

    pub fn find_sequence(&self, pat: &[Quat]) -> Option<usize> {
        if pat.is_empty() || pat.len() > self.len_quats {
            return None;
        }

        let limit = self.len_quats - pat.len();
        'outer: for i in 0..=limit {
            for (j, &expected) in pat.iter().enumerate() {
                if self.get(i + j) != Some(expected) {
                    continue 'outer;
                }
            }
            return Some(i);
        }

        None
    }

    pub fn truncate(&mut self, new_len_quats: usize) {
        if new_len_quats < self.len_quats {
            self.len_quats = new_len_quats;
            let needed_bytes = (new_len_quats + 3) / 4;
            self.data.truncate(needed_bytes);

            // Clean up unused trailing bits in the last byte
            let remainder = new_len_quats % 4;
            if remainder != 0 && !self.data.is_empty() {
                let mask = match remainder {
                    1 => 0b11000000,
                    2 => 0b11110000,
                    3 => 0b11111100,
                    _ => 0xFF,
                };
                let last_idx = self.data.len() - 1;
                self.data[last_idx] &= mask;
            }
        }
    }
}