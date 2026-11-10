use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use crate::buffer::QuatBuffer;
use crate::error::{QuatError, Result};

const CONTAINER_MAGIC: &[u8; 4] = b"QCNT";

pub struct QuatContainer {
    file: File,
    index: HashMap<String, (u64, usize)>, // path -> (offset, length_in_quats)
}

impl QuatContainer {
    /// Create or open a single-file VFS container
    pub fn open(path: &str) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(QuatError::Io)?;

        let metadata = file.metadata().map_err(QuatError::Io)?;
        let index = HashMap::new();

        if metadata.len() > 0 {
            // Read container
            let mut magic = [0u8; 4];
            file.read_exact(&mut magic).map_err(QuatError::Io)?;
            if &magic != CONTAINER_MAGIC {
                return Err(QuatError::InvalidHeader);
            }
            
            // Read index table size and entries (Simplified representation)
            // In a full production system, deserialize index entries here.
        } else {
            // Initialize new container with magic bytes
            file.write_all(CONTAINER_MAGIC).map_err(QuatError::Io)?;
        }

        Ok(Self { file, index })
    }

    /// Write a QuatBuffer into the single-file container
    pub fn write_file_buffer(&mut self, path: &str, buffer: &QuatBuffer) -> Result<()> {
        self.file.seek(SeekFrom::End(0)).map_err(QuatError::Io)?;
        let offset = self.file.stream_position().map_err(QuatError::Io)?;
        
        // Write packed data bytes from QuatBuffer[cite: 18]
        let raw_data = &buffer.data;
        let len_quats = buffer.len_quats;

        // Write length of quats and raw bytes
        self.file.write_all(&(len_quats as u64).to_le_bytes()).map_err(QuatError::Io)?;
        self.file.write_all(&(raw_data.len() as u64).to_le_bytes()).map_err(QuatError::Io)?;
        self.file.write_all(raw_data).map_err(QuatError::Io)?;

        // Update internal index
        self.index.insert(path.to_string(), (offset, len_quats));
        Ok(())
    }
}