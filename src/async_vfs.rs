use tokio::fs;
use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};

pub struct AsyncMemoryQuatVFS {
    // Thread-safe async mutex around storage paths or data buffers
    storage_path: String,
}

impl AsyncMemoryQuatVFS {
    pub fn new(storage_path: &str) -> Self {
        Self {
            storage_path: storage_path.to_string(),
        }
    }

    /// Asynchronously save a QuatBuffer to disk using Tokio
    pub async fn save_to_disk_async(&self, _vfs_path: &str, buffer: &QuatBuffer) -> Result<()> {
        let raw_data = &buffer.data;
        let len_quats = buffer.len_quats;

        let mut out_bytes = Vec::with_capacity(16 + raw_data.len());
        out_bytes.extend_from_slice(b"QVFS");
        out_bytes.extend_from_slice(&(len_quats as u64).to_le_bytes());
        out_bytes.extend_from_slice(raw_data);

        fs::write(&self.storage_path, out_bytes)
            .await
            .map_err(QuatError::Io)?;

        Ok(())
    }

    /// Asynchronously load a QuatBuffer from disk using Tokio
    pub async fn load_from_disk_async(&self) -> Result<QuatBuffer> {
        let bytes = fs::read(&self.storage_path)
            .await
            .map_err(QuatError::Io)?;

        if bytes.len() < 16 {
            return Err(QuatError::TruncatedPayload);
        }

        let len_quats = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let raw_bytes = bytes[16..].to_vec();

        let quat_buffer = QuatBuffer {
            data: raw_bytes,
            len_quats,
        };

        Ok(quat_buffer)
    }
}