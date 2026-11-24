use crate::quat::Quat;
use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};

pub struct DnaStorage;

impl DnaStorage {
    /// Maps a Quaternary value to a DNA nucleotide: Q0->A, Q1->C, Q2->G, Q3->T
    pub fn quat_to_dna(quat: Quat) -> char {
        match quat {
            Quat::Q0 => 'A',
            Quat::Q1 => 'C',
            Quat::Q2 => 'G',
            Quat::Q3 => 'T',
        }
    }

    /// Maps a DNA nucleotide back to a Quaternary value
    pub fn dna_to_quat(c: char) -> Result<Quat> {
        match c.to_ascii_uppercase() {
            'A' => Ok(Quat::Q0),
            'C' => Ok(Quat::Q1),
            'G' => Ok(Quat::Q2),
            'T' => Ok(Quat::Q3),
            _ => Err(QuatError::InvalidHeader),
        }
    }

    /// Converts a full DNA sequence string (e.g., "ACGTACGT") directly into a QuatBuffer
    pub fn sequence_to_buffer(dna_seq: &str) -> Result<QuatBuffer> {
        let mut buffer = QuatBuffer::new();
        for c in dna_seq.chars() {
            let q = Self::dna_to_quat(c)?;
            buffer.push(q);
        }
        Ok(buffer)
    }

    /// Converts a QuatBuffer back into a readable DNA string sequence
    pub fn buffer_to_sequence(buffer: &QuatBuffer) -> String {
        let mut seq = String::with_capacity(buffer.len_quats);
        for i in 0..buffer.len_quats {
            if let Some(q) = buffer.get(i) {
                seq.push(Self::quat_to_dna(q));
            }
        }
        seq
    }
    /// Fast batch conversion from an ASCII DNA byte slice (`b"ACGT..."`) into a `QuatBuffer`.
    /// Processes 4 nucleotides per output byte directly.
    pub fn bytes_to_buffer(dna_bytes: &[u8]) -> Result<QuatBuffer> {
        let mut data = Vec::with_capacity((dna_bytes.len() + 3) / 4);
        let chunks = dna_bytes.chunks_exact(4);
        let tail = chunks.remainder();

        for chunk in chunks {
            let q0 = Self::ascii_to_quat_u8(chunk[0])?;
            let q1 = Self::ascii_to_quat_u8(chunk[1])?;
            let q2 = Self::ascii_to_quat_u8(chunk[2])?;
            let q3 = Self::ascii_to_quat_u8(chunk[3])?;
            data.push((q0 << 6) | (q1 << 4) | (q2 << 2) | q3);
        }

        if !tail.is_empty() {
            let mut last_byte = 0u8;
            for (i, &b) in tail.iter().enumerate() {
                let q = Self::ascii_to_quat_u8(b)?;
                last_byte |= q << (6 - i * 2);
            }
            data.push(last_byte);
        }

        Ok(QuatBuffer {
            data,
            len_quats: dna_bytes.len(),
        })
    }

    /// Fast batch conversion from a `QuatBuffer` directly into an ASCII DNA `String`
    /// using a 4-entry lookup table (`b"ACGT"`).
    pub fn buffer_to_dna_fast(buffer: &QuatBuffer) -> String {
        const LUT: [u8; 4] = [b'A', b'C', b'G', b'T'];
        let mut out = Vec::with_capacity(buffer.len_quats);
        let full_bytes = buffer.len_quats / 4;
        let rem = buffer.len_quats % 4;

        for &byte in &buffer.data[..full_bytes] {
            out.push(LUT[((byte >> 6) & 0b11) as usize]);
            out.push(LUT[((byte >> 4) & 0b11) as usize]);
            out.push(LUT[((byte >> 2) & 0b11) as usize]);
            out.push(LUT[(byte & 0b11) as usize]);
        }

        if rem > 0 {
            let last_byte = buffer.data[full_bytes];
            for i in 0..rem {
                let shift = 6 - (i * 2);
                out.push(LUT[((last_byte >> shift) & 0b11) as usize]);
            }
        }

        // Safety: LUT contains only valid ASCII bytes ('A', 'C', 'G', 'T')
        unsafe { String::from_utf8_unchecked(out) }
    }

    #[inline]
    fn ascii_to_quat_u8(b: u8) -> Result<u8> {
        match b.to_ascii_uppercase() {
            b'A' => Ok(0b00),
            b'C' => Ok(0b01),
            b'G' => Ok(0b10),
            b'T' => Ok(0b11),
            _ => Err(QuatError::InvalidHeader),
        }
    }
}