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
}