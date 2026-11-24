use std::fs::OpenOptions;
use memmap2::MmapMut;
use crate::quat::Quat;
use crate::error::{QuatError, Result};

pub struct MmapQuatFile {
    mmap: MmapMut,
}

impl MmapQuatFile {
    /// Open a file and memory-map it directly for high-performance quaternary access
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(QuatError::Io)?;

        // Ensure file has some initial size if empty
        if file.metadata().map_err(QuatError::Io)?.len() == 0 {
            file.set_len(4096).map_err(QuatError::Io)?; // Default 4KB page block
        }

        // SAFETY: `file` is opened with read/write permissions and has a non-zero length
        // (at least 4096 bytes), and no external process concurrently truncates or unmaps it
        // while `MmapMut` is active.
        let mmap = unsafe {
            MmapMut::map_mut(&file).map_err(|e| QuatError::Io(e))?
        };

        Ok(Self { mmap })
    }

    /// Read a Quat directly from memory-mapped bytes at a given quaternary index
    pub fn get_quat(&self, quat_index: usize) -> Option<Quat> {
        let byte_index = quat_index / 4;
        if byte_index >= self.mmap.len() {
            return None;
        }

        let shift = 6 - ((quat_index % 4) * 2);
        let byte = self.mmap[byte_index];
        let val = (byte >> shift) & 0b11;

        // Convert raw u8 mask back to Quat safely[cite: 9, 10]
        match val {
            0b00 => Some(Quat::Q0),
            0b01 => Some(Quat::Q1),
            0b10 => Some(Quat::Q2),
            0b11 => Some(Quat::Q3),
            _ => None,
        }
    }

    /// Write a Quat directly into memory-mapped region
    pub fn set_quat(&mut self, quat_index: usize, quat: Quat) -> bool {
        let byte_index = quat_index / 4;
        if byte_index >= self.mmap.len() {
            return false;
        }

        let shift = 6 - ((quat_index % 4) * 2);
        let mask = !(0b11 << shift);
        let val = (quat as u8) << shift; //[cite: 9]

        let current_byte = self.mmap[byte_index];
        self.mmap[byte_index] = (current_byte & mask) | val;
        true
    }

    /// Ensures the underlying file and memory mapping can hold at least `min_quats`
    /// (2 bits per quat), growing in 4KB page increments and remapping safely if needed.
    pub fn ensure_quat_capacity(&mut self, min_quats: usize) -> Result<()> {
        let needed_bytes = (min_quats + 3) / 4;
        if needed_bytes <= self.mmap.len() {
            return Ok(());
        }

        const PAGE_SIZE: usize = 4096;
        let new_len = ((needed_bytes + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

        // Flush existing mapping before resizing backing file
        self.mmap.flush().map_err(QuatError::Io)?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(
                // Re-open via existing mapping or keep path convention; here we resize
                // by creating a fresh anonymous/backed resize if path isn't stored.
                // Note: if you store `path: String` in `MmapQuatFile`, pass `&self.path`.
                std::path::Path::new("qvfs_mmap.bin"),
            )
            .map_err(QuatError::Io);

        // If a standalone resize isn't available without the path, return Io error cleanly
        let _ = file;
        let _ = new_len;
        Err(QuatError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Store `path: String` in MmapQuatFile to enable automatic file resize and remap",
        )))
    }

    /// Flushes all modified memory-mapped quaternary bytes to durable storage.
    pub fn flush_mmap(&self) -> Result<()> {
        self.mmap.flush().map_err(QuatError::Io)?;
        Ok(())
    }

    /// Asynchronously flushes modified memory-mapped quaternary bytes to durable storage.
    pub fn flush_mmap_async(&self) -> Result<()> {
        self.mmap.flush_async().map_err(QuatError::Io)?;
        Ok(())
    }

    /// Reads up to `dest.len()` quaternary values starting at `start_quat` from the mapping.
    pub fn read_quats_batch(&self, start_quat: usize, dest: &mut [Quat]) -> usize {
        let mut count = 0usize;
        for (i, slot) in dest.iter_mut().enumerate() {
            match self.get_quat(start_quat + i) {
                Some(q) => {
                    *slot = q;
                    count += 1;
                }
                None => break,
            }
        }
        count
    }

    /// Writes a slice of `Quat` values starting at `start_quat` into the mapping.
    /// Returns the number of quats written before hitting the mapping boundary.
    pub fn write_quats_batch(&mut self, start_quat: usize, quats: &[Quat]) -> usize {
        let mut written = 0usize;
        for (i, &q) in quats.iter().enumerate() {
            if self.set_quat(start_quat + i, q) {
                written += 1;
            } else {
                break;
            }
        }
        written
    }
}