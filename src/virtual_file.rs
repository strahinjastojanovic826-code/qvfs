use std::time::{SystemTime, UNIX_EPOCH};

use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};
use crate::quat::Quat;
use crate::traits::QuatFile;

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone)]
pub struct VirtualQuatFile {
    pub buffer: QuatBuffer,
    pub cursor: usize,
    pub created_at_ms: u64,
    pub modified_at_ms: u64,
}

impl VirtualQuatFile {
    pub fn new() -> Self {
        let now = current_time_ms();
        Self {
            buffer: QuatBuffer::new(),
            cursor: 0,
            created_at_ms: now,
            modified_at_ms: now,
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.buffer.truncate(new_len);
        if self.cursor > new_len {
            self.cursor = new_len;
        }
        self.modified_at_ms = current_time_ms();
    }
}

impl Default for VirtualQuatFile {
    fn default() -> Self {
        Self::new()
    }
}

impl QuatFile for VirtualQuatFile {
    fn read_quat(&mut self) -> Result<Option<Quat>> {
        if let Some(quat) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            Ok(Some(quat))
        } else {
            Ok(None)
        }
    }

    fn write_quat(&mut self, quat: Quat) -> Result<()> {
        if self.cursor < self.buffer.len_quats {
            // Overwrite existing quat
            self.buffer.set(self.cursor, quat);
        } else if self.cursor == self.buffer.len_quats {
            // Append quat
            self.buffer.push(quat);
        } else {
            // Fill space between previous EOF and current cursor position with Q0
            while self.buffer.len_quats < self.cursor {
                self.buffer.push(Quat::Q0);
            }
            self.buffer.push(quat);
        }

        self.cursor += 1;
        self.modified_at_ms = current_time_ms();
        Ok(())
    }

    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize> {
        self.cursor = quat_offset;
        Ok(self.cursor)
    }

    fn len(&self) -> usize {
        self.buffer.len_quats
    }

    fn read_exact_quats(&mut self, buf: &mut [Quat]) -> Result<usize> {
        let mut count = 0;
        for target in buf.iter_mut() {
            if let Some(quat) = self.buffer.get(self.cursor) {
                *target = quat;
                self.cursor += 1;
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }
}