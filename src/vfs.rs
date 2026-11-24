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

pub const QVFS_FORMAT_MAGIC: &[u8; 4] = b"QVFH";
pub const QVFS_FORMAT_VERSION: u16 = 1;
pub const QVFS_FLAG_NONE: u16 = 0;
pub const QVFS_FLAG_RLE_QUATS: u16 = 1 << 0;
pub const QVFS_HEADER_LEN: usize = 24;

/// Header layout (24 bytes, little-endian):
/// [0..4]   magic: "QVFH"
/// [4..6]   version: u16 (1)
/// [6..8]   flags: u16
/// [8..16]  total_quats: u64
/// [16..20] payload_len: u32
/// [20..24] checksum: u32 (checksum over payload bytes)
pub fn encode_versioned_frame(
    quats_packed: &[u8],
    total_quats: u64,
    flags: u16,
) -> Result<Vec<u8>> {
    let payload_len: u32 = quats_packed
        .len()
        .try_into()
        .map_err(|_| QuatError::InvalidHeader)?;

    let mut checksum: u32 = 2166136261;
    for &b in quats_packed {
        checksum ^= b as u32;
        checksum = checksum.wrapping_mul(16777619);
    }

    let mut out = Vec::with_capacity(QVFS_HEADER_LEN + quats_packed.len());
    out.extend_from_slice(QVFS_FORMAT_MAGIC);
    out.extend_from_slice(&QVFS_FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&total_quats.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(quats_packed);
    Ok(out)
}

pub fn decode_versioned_frame(bytes: &[u8]) -> Result<(u16, u64, Vec<u8>)> {
    if bytes.len() < QVFS_HEADER_LEN {
        return Err(QuatError::TruncatedPayload);
    }
    if &bytes[0..4] != QVFS_FORMAT_MAGIC {
        return Err(QuatError::InvalidHeader);
    }

    let version = u16::from_le_bytes(
        bytes[4..6]
            .try_into()
            .map_err(|_| QuatError::InvalidHeader)?,
    );
    if version != QVFS_FORMAT_VERSION {
        return Err(QuatError::InvalidHeader);
    }

    let flags = u16::from_le_bytes(
        bytes[6..8]
            .try_into()
            .map_err(|_| QuatError::InvalidHeader)?,
    );
    let total_quats = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| QuatError::InvalidHeader)?,
    );
    let payload_len = u32::from_le_bytes(
        bytes[16..20]
            .try_into()
            .map_err(|_| QuatError::InvalidHeader)?,
    ) as usize;
    let saved_checksum = u32::from_le_bytes(
        bytes[20..24]
            .try_into()
            .map_err(|_| QuatError::InvalidHeader)?,
    );

    let payload_end = QVFS_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(QuatError::TruncatedPayload)?;
    if bytes.len() < payload_end {
        return Err(QuatError::TruncatedPayload);
    }

    let payload = &bytes[QVFS_HEADER_LEN..payload_end];
    let mut calc_checksum: u32 = 2166136261;
    for &b in payload {
        calc_checksum ^= b as u32;
        calc_checksum = calc_checksum.wrapping_mul(16777619);
    }
    if saved_checksum != calc_checksum {
        return Err(QuatError::InvalidHeader);
    }

    let required_bytes = ((total_quats as usize) + 3) / 4;
    if (flags & QVFS_FLAG_RLE_QUATS) == 0 && payload.len() < required_bytes {
        return Err(QuatError::TruncatedPayload);
    }

    Ok((flags, total_quats, payload.to_vec()))
}

pub type FileHandle = Arc<Mutex<VirtualQuatFile>>;

#[derive(Default)]
pub struct MemoryQuatVFS {
    pub files: RwLock<HashMap<String, FileHandle>>,
}

impl MemoryQuatVFS {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    pub fn save_to_disk(&self, vfs_path: &str, real_disk_path: &str) -> Result<()> {
    let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
    let file_handle = files
        .get(vfs_path)
        .ok_or_else(|| QuatError::FileNotFound(vfs_path.to_string()))?;
    
    let file = file_handle.lock().map_err(|_| QuatError::LockPoisoned)?;

    let total_quats: u64 = file.len() as u64;
    let data_bytes = &file.buffer.data;

    // Simple checksum calculation (FNV-1a style or XOR hash)
    let mut checksum: u32 = 2166136261;
    for &byte in data_bytes {
        checksum ^= byte as u32;
        checksum = checksum.wrapping_mul(16777619);
    }

    let mut out_bytes = Vec::with_capacity(16 + data_bytes.len());
    out_bytes.extend_from_slice(MAGIC_BYTES);
    out_bytes.extend_from_slice(&checksum.to_le_bytes());
    out_bytes.extend_from_slice(&total_quats.to_le_bytes());
    out_bytes.extend_from_slice(data_bytes);

    fs::write(real_disk_path, out_bytes).map_err(QuatError::Io)?;
    Ok(())
}

    pub fn load_from_disk(&self, real_disk_path: &str, vfs_path: &str) -> Result<FileHandle> {
    let bytes = fs::read(real_disk_path).map_err(QuatError::Io)?;
    
    // Minimalna dužina zaglavlja: 4 (magic) + 4 (checksum) + 8 (total_quats) = 16 bajtova
    if bytes.len() < 16 {
        return Err(QuatError::TruncatedPayload);
    }

    if &bytes[0..4] != MAGIC_BYTES {
        return Err(QuatError::InvalidHeader);
    }

    // Čitanje sačuvanog checksum-a sa indeksa [4..8]
    let saved_checksum = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    
    // Čitanje total_quats sa pomjerom na [8..16]
    let total_quats = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let raw_bytes = bytes[16..].to_vec();

    // Izračunavanje checksum-a učitanih podataka radi provjere integriteta
    let mut calc_checksum: u32 = 2166136261;
    for &byte in &raw_bytes {
        calc_checksum ^= byte as u32;
        calc_checksum = calc_checksum.wrapping_mul(16777619);
    }

    if saved_checksum != calc_checksum {
        return Err(QuatError::InvalidHeader); // Fajl je oštećen ili izmijenjen na disku
    }

    let expected_bytes = (total_quats + 3) / 4;
    if raw_bytes.len() < expected_bytes {
        return Err(QuatError::TruncatedPayload);
    }

    let quat_buffer = QuatBuffer {
        data: raw_bytes,
        len_quats: total_quats,
    };

    let virt_file = VirtualQuatFile {
        buffer: quat_buffer,
        cursor: 0,
        created_at_ms: 0,
        modified_at_ms: 0,
    };

    let handle = Arc::new(Mutex::new(virt_file));
    let mut files = self.files.write().map_err(|_| QuatError::LockPoisoned)?;
    files.insert(vfs_path.to_string(), Arc::clone(&handle));

    Ok(handle)
}

/// Saves a virtual file to disk using the unified versioned QVFH v1 format.
    pub fn save_to_disk_v1(&self, vfs_path: &str, real_disk_path: &str) -> Result<()> {
        let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
        let file_handle = files
            .get(vfs_path)
            .ok_or_else(|| QuatError::FileNotFound(vfs_path.to_string()))?;

        let file = file_handle.lock().map_err(|_| QuatError::LockPoisoned)?;
        let total_quats = file.len() as u64;
        let frame = encode_versioned_frame(&file.buffer.data, total_quats, QVFS_FLAG_NONE)?;
        fs::write(real_disk_path, frame).map_err(QuatError::Io)?;
        Ok(())
    }

    /// Loads a virtual file from disk using the unified versioned QVFH v1 format.
    pub fn load_from_disk_v1(&self, real_disk_path: &str, vfs_path: &str) -> Result<FileHandle> {
        let bytes = fs::read(real_disk_path).map_err(QuatError::Io)?;
        let (_flags, total_quats, raw_bytes) = decode_versioned_frame(&bytes)?;

        let quat_buffer = QuatBuffer {
            data: raw_bytes,
            len_quats: total_quats as usize,
        };

        let virt_file = VirtualQuatFile {
            buffer: quat_buffer,
            cursor: 0,
            created_at_ms: 0,
            modified_at_ms: 0,
        };

        let handle = std::sync::Arc::new(std::sync::Mutex::new(virt_file));
        let mut files = self.files.write().map_err(|_| QuatError::LockPoisoned)?;
        files.insert(vfs_path.to_string(), std::sync::Arc::clone(&handle));
        Ok(handle)
    }

pub fn transform_file_logical<F>(&self, path: &str, op: F) -> Result<()>
    where
        F: Fn(Quat) -> Quat,
    {
        let files = self.files.read().map_err(|_| QuatError::LockPoisoned)?;
        let handle = files
            .get(path)
            .ok_or_else(|| QuatError::FileNotFound(path.to_string()))?
            .clone();
        
        drop(files); // Release read lock before acquiring write lock

        let mut file = handle.lock().map_err(|_| QuatError::LockPoisoned)?;
        
        // Transform every quat in the buffer using our new iterator/buffer access
        for i in 0..file.buffer.len_quats {
            if let Some(q) = file.buffer.get(i) {
                let transformed = op(q);
                file.buffer.set(i, transformed);
            }
        }

        file.modified_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(())
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

pub struct QuatIter<'a> {
    buffer: &'a QuatBuffer,
    index: usize,
}

impl<'a> Iterator for QuatIter<'a> {
    type Item = Quat;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.buffer.get(self.index)?;
        self.index += 1;
        Some(val)
    }
}

impl QuatBuffer {
    // Create an immutable iterator over the buffer
    pub fn iter(&self) -> QuatIter<'_> {
        QuatIter {
            buffer: self,
            index: 0,
        }
    }
}

// IntoIterator for owned QuatBuffer
pub struct QuatIntoIter {
    buffer: QuatBuffer,
    index: usize,
}

impl Iterator for QuatIntoIter {
    type Item = Quat;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.buffer.get(self.index)?;
        self.index += 1;
        Some(val)
    }
}

impl IntoIterator for QuatBuffer {
    type Item = Quat;
    type IntoIter = QuatIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        QuatIntoIter {
            buffer: self,
            index: 0,
        }
    }
}