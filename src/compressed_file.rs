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
        let mut count = 0;
        for target in buf.iter_mut() {
            if let Some(quat) = self.raw_buffer.get(self.cursor) {
                *target = quat;
                self.cursor += 1;
                count += 1;
            } else {
                break;
            }
        }
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