use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};
use crate::quat::Quat;
use crate::traits::QuatFile;

#[derive(Debug, Clone)]
pub struct CompressedQuatFile {
    pub raw_buffer: QuatBuffer,
    pub cursor: usize,
}

impl CompressedQuatFile {
    pub fn new(compressed_buffer: QuatBuffer) -> Self {
        Self {
            raw_buffer: compressed_buffer,
            cursor: 0,
        }
    }

    /// Splits a `QuatBuffer` into fixed-size quaternary segments (e.g., 4096 quats per segment).
    pub fn segment_quats(buffer: &QuatBuffer, segment_size_quats: usize) -> Vec<QuatBuffer> {
        if segment_size_quats == 0 || buffer.len_quats == 0 {
            return Vec::new();
        }

        let num_segments = (buffer.len_quats + segment_size_quats - 1) / segment_size_quats;
        let mut segments = Vec::with_capacity(num_segments);
        let mut offset = 0usize;

        while offset < buffer.len_quats {
            let count = segment_size_quats.min(buffer.len_quats - offset);
            let mut seg = QuatBuffer::with_capacity(count);
            for i in 0..count {
                if let Some(q) = buffer.get(offset + i) {
                    seg.push(q);
                }
            }
            segments.push(seg);
            offset += count;
        }

        segments
    }

    /// Compresses a `QuatBuffer` into a compact RLE byte stream of `(quat_2bit, run_len_u16_le)` pairs.
    /// Ideal for repetitive quaternary runs (e.g., homopolymer stretches or zero-padded regions).
    pub fn compress_rle_quats(buffer: &QuatBuffer) -> Vec<u8> {
        if buffer.len_quats == 0 {
            return Vec::new();
        }

        let mut encoded = Vec::new();
        let mut current_q = buffer.get(0).unwrap_or(Quat::Q0);
        let mut run_len: u16 = 1;

        for i in 1..buffer.len_quats {
            let q = buffer.get(i).unwrap_or(Quat::Q0);
            if q == current_q && run_len < u16::MAX {
                run_len += 1;
            } else {
                encoded.push(current_q as u8);
                encoded.extend_from_slice(&run_len.to_le_bytes());
                current_q = q;
                run_len = 1;
            }
        }

        encoded.push(current_q as u8);
        encoded.extend_from_slice(&run_len.to_le_bytes());
        encoded
    }

    /// Decompresses RLE-encoded quaternary bytes produced by `compress_rle_quats` back into a `QuatBuffer`.
    pub fn decompress_rle_quats(rle_bytes: &[u8], expected_quats: usize) -> Result<QuatBuffer> {
        if rle_bytes.len() % 3 != 0 {
            return Err(QuatError::TruncatedPayload);
        }

        let mut out = QuatBuffer::with_capacity(expected_quats);
        let mut idx = 0usize;

        while idx < rle_bytes.len() {
            let q = match rle_bytes[idx] & 0b11 {
                0 => Quat::Q0,
                1 => Quat::Q1,
                2 => Quat::Q2,
                _ => Quat::Q3,
            };
            let run_len = u16::from_le_bytes(
                rle_bytes[idx + 1..idx + 3]
                    .try_into()
                    .map_err(|_| QuatError::TruncatedPayload)?,
            ) as usize;

            for _ in 0..run_len {
                if out.len_quats >= expected_quats {
                    break;
                }
                out.push(q);
            }

            idx += 3;
        }

        Ok(out)
    }

}

impl QuatFile for CompressedQuatFile {
    fn read_quat(&mut self) -> Result<Option<Quat>> {
        if let Some(quat) = self.raw_buffer.get(self.cursor) {
            self.cursor += 1;
            Ok(Some(quat))
        } else {
            Ok(None)
        }
    }

    fn write_quat(&mut self, _quat: Quat) -> Result<()> {
        Err(QuatError::PermissionDenied(
            "CompressedQuatFile is read-only. Decompress first to modify.".to_string(),
        ))
    }

    fn read_exact_quats(&mut self, buf: &mut [Quat]) -> Result<usize> {
        let count = self.raw_buffer.unpack_into_slice(self.cursor, buf);
        self.cursor += count;
        Ok(count)
    }

    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize> {
        self.cursor = quat_offset;
        Ok(self.cursor)
    }

    fn len(&self) -> usize {
        self.raw_buffer.len_quats
    }

}