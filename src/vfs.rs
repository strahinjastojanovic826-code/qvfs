use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, RwLock};
use crate::QuatFile;
use crate::Quat;

use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};
use crate::traits::QuatFileSystem;
use crate::virtual_file::VirtualQuatFile;

const MAGIC_BYTES: &[u8; 4] = b"QVFS";

pub type FileHandle = Arc<Mutex<VirtualQuatFile>>;

#[derive(Default)]
pub struct MemoryQuatVFS {
    files: RwLock<HashMap<String, FileHandle>>,
}

impl MemoryQuatVFS {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    pub fn save_to_disk(&self, vfs_path: &str, real_disk_path: &str) -> Result<()> {
        let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
        let handle = files
            .get(vfs_path)
            .ok_or_else(|| QuatError::FileNotFound(vfs_path.to_string()))?;

        let file = handle.lock().map_err(|_| QuatError::LockPoisoned)?;

        let mut out_bytes = Vec::with_capacity(12 + file.buffer.data.len());
        out_bytes.extend_from_slice(MAGIC_BYTES);

        let total_quats = file.buffer.len_quats as u64;
        out_bytes.extend_from_slice(&total_quats.to_le_bytes());
        out_bytes.extend_from_slice(&file.buffer.data);

        fs::write(real_disk_path, out_bytes)?;
        Ok(())
    }

    pub fn load_from_disk(&self, real_disk_path: &str, vfs_path: &str) -> Result<FileHandle> {
        let bytes = fs::read(real_disk_path)?;

        if bytes.len() < 12 {
            return Err(QuatError::TruncatedPayload);
        }

        if &bytes[0..4] != MAGIC_BYTES {
            return Err(QuatError::InvalidHeader);
        }

        let total_quats = u64::from_le_bytes(bytes[4..12].try_into().unwrap()) as usize;
        let raw_bytes = bytes[12..].to_vec();

        let expected_bytes = (total_quats + 3) / 4;
        if raw_bytes.len() < expected_bytes {
            return Err(QuatError::TruncatedPayload);
        }

        let quat_buffer = QuatBuffer {
            data: raw_bytes,
            len_quats: total_quats,
        };

        let mut virt_file = VirtualQuatFile::new();
        virt_file.buffer = quat_buffer;

        let handle = Arc::new(Mutex::new(virt_file));
        let mut files = self.files.write().map_err(|_| QuatError::LockPoisoned)?;
        files.insert(vfs_path.to_string(), Arc::clone(&handle));

        Ok(handle)
    }

    pub fn list_dir(&self, prefix: &str) -> Result<Vec<String>> {
        let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
        Ok(files
            .keys()
            .filter(|path| path.starts_with(prefix))
            .cloned()
            .collect())
    }
}

impl QuatFileSystem for MemoryQuatVFS {
    type FileHandle = FileHandle;

    fn create(&mut self, path: &str) -> Result<Self::FileHandle> {
        let virt_file = VirtualQuatFile::new();
        let handle = Arc::new(Mutex::new(virt_file));

        let mut files = self.files.write().map_err(|_| QuatError::LockPoisoned)?;
        files.insert(path.to_string(), Arc::clone(&handle));

        Ok(handle)
    }

    fn open(&mut self, path: &str) -> Result<Self::FileHandle> {
        let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| QuatError::FileNotFound(path.to_string()))
    }

    fn delete(&mut self, path: &str) -> Result<()> {
        let mut files = self.files.write().map_err(|_| QuatError::LockPoisoned)?;
        if files.remove(path).is_some() {
            Ok(())
        } else {
            Err(QuatError::FileNotFound(path.to_string()))
        }
    }
}

// Implement QuatFile wrapper directly on Thread-Safe Handles
impl QuatFile for FileHandle {
    fn read_quat(&mut self) -> Result<Option<Quat>> {
        self.lock()
            .map_err(|_| QuatError::LockPoisoned)?
            .read_quat()
    }

    fn write_quat(&mut self, quat: Quat) -> Result<()> {
        self.lock()
            .map_err(|_| QuatError::LockPoisoned)?
            .write_quat(quat)
    }

    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize> {
        self.lock()
            .map_err(|_| QuatError::LockPoisoned)?
            .seek_quat(quat_offset)
    }

    fn len(&self) -> usize {
        self.lock().map(|f| f.len()).unwrap_or(0)
    }

    fn read_exact_quats(&mut self, buf: &mut [Quat]) -> Result<usize> {
        self.lock()
            .map_err(|_| QuatError::LockPoisoned)?
            .read_exact_quats(buf)
    }
}