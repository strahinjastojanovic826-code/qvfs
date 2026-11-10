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
}