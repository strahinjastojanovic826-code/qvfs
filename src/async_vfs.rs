use tokio::fs;
use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};
use crate::Quat;

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

    /// Asynchronously persists a `QuatBuffer` to an explicit `target_path` using Tokio FS.
    pub async fn save_buffer_to_path_async(
        &self,
        target_path: &str,
        buffer: &QuatBuffer,
    ) -> Result<()> {
        let raw_data = &buffer.data;
        let len_quats = buffer.len_quats as u64;

        let mut out_bytes = Vec::with_capacity(12 + raw_data.len());
        out_bytes.extend_from_slice(b"QVFS");
        out_bytes.extend_from_slice(&len_quats.to_le_bytes());
        out_bytes.extend_from_slice(raw_data);

        fs::write(target_path, out_bytes)
            .await
            .map_err(QuatError::Io)?;
        Ok(())
    }

    /// Asynchronously loads a `QuatBuffer` from an explicit `source_path` using Tokio FS.
    pub async fn load_buffer_from_path_async(&self, source_path: &str) -> Result<QuatBuffer> {
        let bytes = fs::read(source_path).await.map_err(QuatError::Io)?;
        if bytes.len() < 12 {
            return Err(QuatError::TruncatedPayload);
        }
        if &bytes[0..4] != b"QVFS" {
            return Err(QuatError::InvalidHeader);
        }

        let len_quats = u64::from_le_bytes(
            bytes[4..12]
                .try_into()
                .map_err(|_| QuatError::InvalidHeader)?,
        ) as usize;

        let expected_bytes = (len_quats + 3) / 4;
        let raw_bytes = bytes[12..].to_vec();
        if raw_bytes.len() < expected_bytes {
            return Err(QuatError::TruncatedPayload);
        }

        Ok(QuatBuffer {
            data: raw_bytes,
            len_quats,
        })
    }

    /// Offloads a blocking `QuatBuffer` transformation to Tokio's blocking thread pool.
    pub async fn transform_buffer_blocking<F>(
        mut buffer: QuatBuffer,
        op: F,
    ) -> Result<QuatBuffer>
    where
        F: Fn(Quat) -> Quat + Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            for i in 0..buffer.len_quats {
                if let Some(q) = buffer.get(i) {
                    buffer.set(i, op(q));
                }
            }
            buffer
        })
        .await
        .map_err(|_| QuatError::LockPoisoned)
    }

}