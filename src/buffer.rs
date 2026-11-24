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
    /// Batch-packs a slice of `Quat` values into the buffer using 4-at-a-time chunking.
    /// Avoids per-element bit-shifting overhead when the buffer is byte-aligned.
    pub fn extend_from_quats(&mut self, quats: &[Quat]) {
        if quats.is_empty() {
            return;
        }

        // Align cursor to a byte boundary first if currently unaligned
        let unaligned = (4 - (self.len_quats % 4)) % 4;
        let split_idx = unaligned.min(quats.len());
        for &q in &quats[..split_idx] {
            self.push(q);
        }

        let remaining = &quats[split_idx..];
        if remaining.is_empty() {
            return;
        }

        // Now self.len_quats % 4 == 0: pack 4 quats -> 1 byte directly
        let full_chunks = remaining.chunks_exact(4);
        let tail = full_chunks.remainder();

        self.data.reserve((remaining.len() + 3) / 4);
        for chunk in full_chunks {
            let packed = ((chunk[0] as u8) << 6)
                | ((chunk[1] as u8) << 4)
                | ((chunk[2] as u8) << 2)
                | (chunk[3] as u8);
            self.data.push(packed);
        }
        self.len_quats += remaining.len() - tail.len();

        for &q in tail {
            self.push(q);
        }
    }

    /// Unpacks the entire buffer into a flat `Vec<Quat>` using 4-way unrolled byte expansion.
    pub fn to_quat_vec(&self) -> Vec<Quat> {
        let mut out = Vec::with_capacity(self.len_quats);
        let full_bytes = self.len_quats / 4;
        let remainder = self.len_quats % 4;

        for &byte in &self.data[..full_bytes] {
            out.extend_from_slice(&Quat::unpack_4(byte));
        }

        if remainder > 0 {
            let last_byte = self.data[full_bytes];
            for i in 0..remainder {
                let shift = 6 - (i * 2);
                out.push(Quat::from_u8_masked((last_byte >> shift) & 0b11));
            }
        }

        out
    }

    /// Unpacks a range `[start..start+dest.len()]` directly into a caller-provided slice.
    /// Returns the number of unpacked quats.
    pub fn unpack_into_slice(&self, start: usize, dest: &mut [Quat]) -> usize {
        if start >= self.len_quats || dest.is_empty() {
            return 0;
        }

        let count = dest.len().min(self.len_quats - start);
        let mut out_idx = 0;
        let mut q_idx = start;

        // 1. Handle leading unaligned quats
        while out_idx < count && (q_idx % 4) != 0 {
            if let Some(q) = self.get(q_idx) {
                dest[out_idx] = q;
            }
            out_idx += 1;
            q_idx += 1;
        }

        // 2. Fast aligned byte-to-4-quats loop
        let aligned_full_bytes = (count - out_idx) / 4;
        let start_byte = q_idx / 4;
        for b_offset in 0..aligned_full_bytes {
            let unpacked = Quat::unpack_4(self.data[start_byte + b_offset]);
            dest[out_idx..out_idx + 4].copy_from_slice(&unpacked);
            out_idx += 4;
            q_idx += 4;
        }

        // 3. Trailing unaligned quats
        while out_idx < count {
            if let Some(q) = self.get(q_idx) {
                dest[out_idx] = q;
            }
            out_idx += 1;
            q_idx += 1;
        }

        count
    }

    /// SWAR: Counts occurrences of all 4 quaternary states `[Q0, Q1, Q2, Q3]` in a single pass.
    /// Processes 32 quats (64 bits) per iteration using bit-parallel popcount.
    pub fn count_states(&self) -> [usize; 4] {
        if self.len_quats == 0 {
            return [0; 4];
        }

        let mut counts = [0usize; 4];
        let full_bytes = self.len_quats / 4;
        let aligned_data = &self.data[..full_bytes];

        // Process 8 bytes (32 quats) per iteration using u64 SWAR bit-slicing
        let chunks = aligned_data.chunks_exact(8);
        let byte_tail = chunks.remainder();

        for chunk in chunks {
            let word = u64::from_ne_bytes(chunk.try_into().unwrap());
            // Low bit of each 2-bit pair (mask 0x5555... = 0b0101...)
            let lo = word & 0x5555_5555_5555_5555;
            // High bit of each 2-bit pair shifted to bit 0
            let hi = (word >> 1) & 0x5555_5555_5555_5555;

            let q3 = (hi & lo).count_ones() as usize;       // 11
            let q2 = (hi & !lo).count_ones() as usize;      // 10
            let q1 = (!hi & lo).count_ones() as usize;      // 01
            let q0 = (!hi & !lo).count_ones() as usize;     // 00

            counts[0] += q0;
            counts[1] += q1;
            counts[2] += q2;
            counts[3] += q3;
        }

        // Remaining full bytes
        for &byte in byte_tail {
            for q in Quat::unpack_4(byte) {
                counts[q as usize] += 1;
            }
        }

        // Trailing partial byte
        let rem = self.len_quats % 4;
        if rem > 0 {
            let last_byte = self.data[full_bytes];
            for i in 0..rem {
                let shift = 6 - (i * 2);
                let q = (last_byte >> shift) & 0b11;
                counts[q as usize] += 1;
            }
        }

        counts
    }

    /// SWAR: In-place bitwise quaternary complement (Q0<->Q3, Q1<->Q2) across 32 quats at a time.
    pub fn complement_in_place(&mut self) {
        if self.len_quats == 0 {
            return;
        }

        for chunk in self.data.chunks_exact_mut(8) {
            let mut word = u64::from_ne_bytes(chunk.try_into().unwrap());
            word = !word;
            chunk.copy_from_slice(&word.to_ne_bytes());
        }

        let rem_bytes = self.data.len() % 8;
        let start = self.data.len() - rem_bytes;
        for b in &mut self.data[start..] {
            *b = !*b;
        }

        self.clear_trailing_bits();
    }

    /// Returns a new buffer containing the reverse complement of this quaternary sequence.
    /// Reverses quat order and applies complement (A<->T, C<->G) in one pass.
    pub fn reverse_complement(&self) -> Self {
        let mut out = Self::with_capacity(self.len_quats);
        for i in (0..self.len_quats).rev() {
            if let Some(q) = self.get(i) {
                out.push(q.complement());
            }
        }
        out
    }

    /// SWAR: Element-wise quaternary AND between two buffers of equal length (8 bytes / 32 quats per cycle).
    pub fn batch_and(&self, other: &Self) -> Option<Self> {
        if self.len_quats != other.len_quats {
            return None;
        }
        let mut data = vec![0u8; self.data.len()];
        for ((dst, &a), &b) in data.iter_mut().zip(&self.data).zip(&other.data) {
            *dst = a & b;
        }
        let mut res = Self {
            data,
            len_quats: self.len_quats,
        };
        res.clear_trailing_bits();
        Some(res)
    }

    /// SWAR: Element-wise quaternary OR between two buffers of equal length.
    pub fn batch_or(&self, other: &Self) -> Option<Self> {
        if self.len_quats != other.len_quats {
            return None;
        }
        let mut data = vec![0u8; self.data.len()];
        for ((dst, &a), &b) in data.iter_mut().zip(&self.data).zip(&other.data) {
            *dst = a | b;
        }
        let mut res = Self {
            data,
            len_quats: self.len_quats,
        };
        res.clear_trailing_bits();
        Some(res)
    }

    /// SWAR: Element-wise quaternary XOR between two buffers of equal length.
    pub fn batch_xor(&self, other: &Self) -> Option<Self> {
        if self.len_quats != other.len_quats {
            return None;
        }
        let mut data = vec![0u8; self.data.len()];
        for ((dst, &a), &b) in data.iter_mut().zip(&self.data).zip(&other.data) {
            *dst = a ^ b;
        }
        let mut res = Self {
            data,
            len_quats: self.len_quats,
        };
        res.clear_trailing_bits();
        Some(res)
    }

    /// SWAR: Element-wise modulo-4 addition `(a + b) mod 4` across 32 quats per `u64` word.
    /// Prevents carry propagation across 2-bit boundaries using even-bit carry suppression.
    pub fn batch_add_mod4(&self, other: &Self) -> Option<Self> {
        if self.len_quats != other.len_quats {
            return None;
        }

        const LO_MASK: u64 = 0x5555_5555_5555_5555; // bit 0 of each 2-bit lane
        const HI_MASK: u64 = 0xAAAA_AAAA_AAAA_AAAA; // bit 1 of each 2-bit lane

        let mut data = vec![0u8; self.data.len()];
        let mut dst_chunks = data.chunks_exact_mut(8);
        let mut a_chunks = self.data.chunks_exact(8);
        let mut b_chunks = other.data.chunks_exact(8);

        for ((d, a), b) in (&mut dst_chunks).zip(&mut a_chunks).zip(&mut b_chunks) {
            let wa = u64::from_ne_bytes(a.try_into().unwrap());
            let wb = u64::from_ne_bytes(b.try_into().unwrap());

            let sum_lo = (wa & LO_MASK) + (wb & LO_MASK);
            let sum_hi = (wa ^ wb) & HI_MASK;
            let res_word = sum_lo ^ sum_hi;

            d.copy_from_slice(&res_word.to_ne_bytes());
        }

        for ((d, &a), &b) in dst_chunks
            .into_remainder()
            .iter_mut()
            .zip(a_chunks.remainder())
            .zip(b_chunks.remainder())
        {
            const LO_B: u8 = 0x55;
            const HI_B: u8 = 0xAA;
            *d = ((a & LO_B).wrapping_add(b & LO_B)) ^ ((a ^ b) & HI_B);
        }

        let mut res = Self {
            data,
            len_quats: self.len_quats,
        };
        res.clear_trailing_bits();
        Some(res)
    }

    /// SWAR: Hamming distance (number of differing quaternary positions) between two equal-length buffers.
    /// Processes 32 quats per `u64` word without unpacking.
    pub fn hamming_distance(&self, other: &Self) -> Option<usize> {
        if self.len_quats != other.len_quats {
            return None;
        }
        if self.len_quats == 0 {
            return Some(0);
        }

        const LO_MASK: u64 = 0x5555_5555_5555_5555;
        let full_bytes = self.len_quats / 4;
        let mut dist = 0usize;

        let mut a_chunks = self.data[..full_bytes].chunks_exact(8);
        let mut b_chunks = other.data[..full_bytes].chunks_exact(8);

        for (a, b) in (&mut a_chunks).zip(&mut b_chunks) {
            let diff = u64::from_ne_bytes(a.try_into().unwrap())
                ^ u64::from_ne_bytes(b.try_into().unwrap());
            // Collapse each 2-bit lane into bit 0: 1 if either bit differs
            let any_diff = (diff | (diff >> 1)) & LO_MASK;
            dist += any_diff.count_ones() as usize;
        }

        for (&a, &b) in a_chunks.remainder().iter().zip(b_chunks.remainder()) {
            let diff = a ^ b;
            let any_diff = (diff | (diff >> 1)) & 0x55;
            dist += any_diff.count_ones() as usize;
        }

        let rem = self.len_quats % 4;
        if rem > 0 {
            let diff = self.data[full_bytes] ^ other.data[full_bytes];
            for i in 0..rem {
                let shift = 6 - (i * 2);
                if ((diff >> shift) & 0b11) != 0 {
                    dist += 1;
                }
            }
        }

        Some(dist)
    }

    /// Ensures unused trailing bits in the final byte remain zeroed.
    #[inline]
    fn clear_trailing_bits(&mut self) {
        let remainder = self.len_quats % 4;
        if remainder != 0 && !self.data.is_empty() {
            let mask = match remainder {
                1 => 0b1100_0000,
                2 => 0b1111_0000,
                3 => 0b1111_1100,
                _ => 0xFF,
            };
            let last_idx = self.data.len() - 1;
            self.data[last_idx] &= mask;
        }
    }
}