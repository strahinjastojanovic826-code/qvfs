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

    /// Writes a transaction begin marker (op_type = 0).
    pub fn log_begin(&mut self) -> Result<()> {
        let op_type: u8 = 0;
        self.wal_file.write_all(&[op_type]).map_err(QuatError::Io)?;
        self.wal_file.flush().map_err(QuatError::Io)?;
        Ok(())
    }

    /// Writes a transaction abort marker (op_type = 3) and flushes to disk.
    pub fn log_abort(&mut self) -> Result<()> {
        let op_type: u8 = 3;
        self.wal_file.write_all(&[op_type]).map_err(QuatError::Io)?;
        self.wal_file.sync_all().map_err(QuatError::Io)?;
        Ok(())
    }

    /// Commits pending WAL entries durably (`sync_all`) and truncates the WAL file.
    pub fn commit_and_truncate(&mut self) -> Result<()> {
        let op_type: u8 = 2;
        self.wal_file.write_all(&[op_type]).map_err(QuatError::Io)?;
        self.wal_file.sync_all().map_err(QuatError::Io)?;
        self.wal_file.set_len(0).map_err(QuatError::Io)?;
        self.wal_file.sync_all().map_err(QuatError::Io)?;
        Ok(())
    }

    /// Replays committed quaternary write records from the WAL into a target `QuatBuffer`.
    /// Uncommitted records without a Commit (2) marker are ignored.
    pub fn replay_committed(&mut self, target: &mut crate::buffer::QuatBuffer) -> Result<usize> {
        use std::io::Seek;
        self.wal_file.seek(std::io::SeekFrom::Start(0)).map_err(QuatError::Io)?;

        let mut pending_writes: Vec<(usize, Quat)> = Vec::new();
        let mut applied_count = 0usize;

        loop {
            let mut op_buf = [0u8; 1];
            match self.wal_file.read_exact(&mut op_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(QuatError::Io(e)),
            }

            match op_buf[0] {
                0 => {
                    // BeginTransaction
                    pending_writes.clear();
                }
                1 => {
                    // WriteQuat: index (u64 LE) + val (u8)
                    let mut idx_buf = [0u8; 8];
                    let mut val_buf = [0u8; 1];
                    self.wal_file.read_exact(&mut idx_buf).map_err(QuatError::Io)?;
                    self.wal_file.read_exact(&mut val_buf).map_err(QuatError::Io)?;
                    let index = u64::from_le_bytes(idx_buf) as usize;
                    let quat = match val_buf[0] & 0b11 {
                        0 => Quat::Q0,
                        1 => Quat::Q1,
                        2 => Quat::Q2,
                        _ => Quat::Q3,
                    };
                    pending_writes.push((index, quat));
                }
                2 => {
                    // Commit: apply all pending writes
                    for (index, quat) in pending_writes.drain(..) {
                        while target.len_quats <= index {
                            target.push(Quat::Q0);
                        }
                        target.set(index, quat);
                        applied_count += 1;
                    }
                }
                3 => {
                    // Abort: discard pending writes
                    pending_writes.clear();
                }
                _ => return Err(QuatError::InvalidHeader),
            }
        }

        Ok(applied_count)
    }

    /// Atomically writes bytes to `target_path` by writing to a temp file, syncing, and renaming.
    pub fn atomic_write_file(target_path: &str, bytes: &[u8]) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let tmp_path = format!("{}.tmp", target_path);
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(QuatError::Io)?;

        tmp_file.write_all(bytes).map_err(QuatError::Io)?;
        tmp_file.sync_all().map_err(QuatError::Io)?;
        drop(tmp_file);

        std::fs::rename(&tmp_path, target_path).map_err(QuatError::Io)?;
        Ok(())
    }
    
}