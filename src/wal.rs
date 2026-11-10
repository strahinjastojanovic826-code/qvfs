use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use crate::quat::Quat;
use crate::error::{QuatError, Result};

#[derive(Debug)]
pub enum WalOperation {
    BeginTransaction,
    WriteQuat { index: usize, val: u8 },
    Commit,
    Abort,
}

pub struct WriteAheadLog {
    wal_file: File,
}

impl WriteAheadLog {
    /// Initialize or open a WAL file alongside the database
    pub fn new(path: &str) -> Result<Self> {
        let wal_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(QuatError::Io)?;

        Ok(Self { wal_file })
    }

    /// Log an operation before executing it in memory/disk
    pub fn log_write(&mut self, index: usize, quat: Quat) -> Result<()> {
        let op_type: u8 = 1; // 1 = WriteQuat
        let val = quat as u8; //[cite: 9]

        self.wal_file.write_all(&[op_type]).map_err(QuatError::Io)?;
        self.wal_file.write_all(&(index as u64).to_le_bytes()).map_err(QuatError::Io)?;
        self.wal_file.write_all(&[val]).map_err(QuatError::Io)?;
        self.wal_file.flush().map_err(QuatError::Io)?;
        
        Ok(())
    }

    /// Commit transaction and clear/truncate WAL
    pub fn commit(&mut self) -> Result<()> {
        let op_type: u8 = 2; // 2 = Commit
        self.wal_file.write_all(&[op_type]).map_err(QuatError::Io)?;
        self.wal_file.flush().map_err(QuatError::Io)?;
        Ok(())
    }
}