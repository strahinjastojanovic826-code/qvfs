use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::sync::{Arc, Mutex};

// ==========================================
// 1. TIPOVI DATA I KONVERZIJE (Quat & DNA)
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Quat {
    Q0 = 0b00, // 0 - Adenin  (A)
    Q1 = 0b01, // 1 - Citozin (C)
    Q2 = 0b10, // 2 - Guanin  (G)
    Q3 = 0b11, // 3 - Timin   (T)
}

impl Quat {
    pub fn from_u8(val: u8) -> Self {
        match val & 0b11 {
            0b00 => Quat::Q0,
            0b01 => Quat::Q1,
            0b10 => Quat::Q2,
            _ => Quat::Q3,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'A' => Some(Quat::Q0),
            'C' => Some(Quat::Q1),
            'G' => Some(Quat::Q2),
            'T' => Some(Quat::Q3),
            _ => None,
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Quat::Q0 => 'A',
            Quat::Q1 => 'C',
            Quat::Q2 => 'G',
            Quat::Q3 => 'T',
        }
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self, self.as_char())
    }
}

// ==========================================
// 2. OPTIMIZOVANI QuatBuffer (Sa podrškom za overwrite)
// ==========================================

#[derive(Debug, Clone, Default)]
pub struct QuatBuffer {
    data: Vec<u8>,
    len_quats: usize,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len_quats: 0,
        }
    }

    pub fn push(&mut self, quat: Quat) {
        let byte_idx = self.len_quats / 4;
        let shift = (self.len_quats % 4) * 2;

        if byte_idx >= self.data.len() {
            self.data.push(0);
        }

        self.data[byte_idx] &= !(0b11 << shift);
        self.data[byte_idx] |= quat.as_u8() << shift;
        self.len_quats += 1;
    }

    /// Izmena postojećeg kvata na specifičnom indeksu
    pub fn set(&mut self, index: usize, quat: Quat) -> bool {
        if index >= self.len_quats {
            return false;
        }

        let byte_idx = index / 4;
        let shift = (index % 4) * 2;

        self.data[byte_idx] &= !(0b11 << shift);
        self.data[byte_idx] |= quat.as_u8() << shift;
        true
    }

    pub fn get(&self, index: usize) -> Option<Quat> {
        if index >= self.len_quats {
            return None;
        }

        let byte_idx = index / 4;
        let shift = (index % 4) * 2;

        let raw = (self.data[byte_idx] >> shift) & 0b11;
        Some(Quat::from_u8(raw))
    }

    pub fn len(&self) -> usize {
        self.len_quats
    }

    pub fn is_empty(&self) -> bool {
        self.len_quats == 0
    }
}

// ==========================================
// 3. VFS TRAIT INTERFEJS
// ==========================================

pub trait QuatFile {
    fn read_quat(&mut self) -> Result<Option<Quat>>;
    fn write_quat(&mut self, quat: Quat) -> Result<()>;
    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize>;
    fn len(&self) -> usize;
}

pub trait QuatFileSystem {
    type FileHandle;

    fn open(&mut self, path: &str) -> Result<Self::FileHandle>;
    fn create(&mut self, path: &str) -> Result<Self::FileHandle>;
    fn delete(&mut self, path: &str) -> Result<()>;
}

// ==========================================
// 4. IN-MEMORY VFS IMPLEMENTACIJA
// ==========================================

#[derive(Debug, Clone, Default)]
pub struct VirtualQuatFile {
    buffer: QuatBuffer,
    cursor: usize,
}

impl VirtualQuatFile {
    pub fn new() -> Self {
        Self {
            buffer: QuatBuffer::new(),
            cursor: 0,
        }
    }
}

impl QuatFile for VirtualQuatFile {
    fn read_quat(&mut self) -> Result<Option<Quat>> {
        let quat = self.buffer.get(self.cursor);
        if quat.is_some() {
            self.cursor += 1;
        }
        Ok(quat)
    }

    fn write_quat(&mut self, quat: Quat) -> Result<()> {
        if self.cursor < self.buffer.len() {
            // Overwrite postojećeg kvata na poziciji kursora
            self.buffer.set(self.cursor, quat);
        } else {
            // Dopisivanje na kraj
            self.buffer.push(quat);
        }
        self.cursor += 1;
        Ok(())
    }

    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize> {
        if quat_offset <= self.buffer.len() {
            self.cursor = quat_offset;
            Ok(self.cursor)
        } else {
            Err(Error::new(ErrorKind::UnexpectedEof, "Seek beyond file boundaries"))
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

// Nitno-bezbedni pokazivač na deljeni fajl (Arc + Mutex umesto Rc + RefCell)
pub type FileHandle = Arc<Mutex<VirtualQuatFile>>;

pub struct MemoryQuatVFS {
    files: HashMap<String, FileHandle>,
}

impl MemoryQuatVFS {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn save_to_disk(&self, vfs_path: &str, real_disk_path: &str) -> Result<()> {
        if let Some(file_handle) = self.files.get(vfs_path) {
            let file = file_handle.lock().map_err(|_| {
                Error::new(ErrorKind::Other, "Thread locking problem (Mutex poison)")
            })?;
            std::fs::write(real_disk_path, &file.buffer.data)?;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "The file does not exist in VFS"))
        }
    }

    pub fn load_from_disk(&mut self, real_disk_path: &str, vfs_path: &str) -> Result<FileHandle> {
        let bytes = std::fs::read(real_disk_path)?;
        let mut quat_buffer = QuatBuffer::new();

        for byte in bytes {
            for shift in (0..4).map(|i| i * 2) {
                let q_val = (byte >> shift) & 0b11;
                quat_buffer.push(Quat::from_u8(q_val));
            }
        }

        let virt_file = VirtualQuatFile {
            buffer: quat_buffer,
            cursor: 0,
        };

        let handle = Arc::new(Mutex::new(virt_file));
        self.files.insert(vfs_path.to_string(), Arc::clone(&handle));
        Ok(handle)
    }
}

impl QuatFileSystem for MemoryQuatVFS {
    type FileHandle = FileHandle;

    fn open(&mut self, path: &str) -> Result<Self::FileHandle> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "File not found"))
    }

    fn create(&mut self, path: &str) -> Result<Self::FileHandle> {
        let file = Arc::new(Mutex::new(VirtualQuatFile::new()));
        self.files.insert(path.to_string(), Arc::clone(&file));
        Ok(file)
    }

    fn delete(&mut self, path: &str) -> Result<()> {
        if self.files.remove(path).is_some() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "File not found"))
        }
    }
}

// ==========================================
// 5. TEST OVERWRITE I VFS FUNKCIONALNOSTI
// ==========================================

fn main() -> Result<()> {
    println!("=== Testing the Fixed Quat VFS Library ===\n");

    let mut vfs = MemoryQuatVFS::new();
    let file_handle = vfs.create("/test.quat")?;

    // 1. Upisivanje niza "AAAA"
    {
        let mut file = file_handle.lock().unwrap();
        for _ in 0..4 {
            file.write_quat(Quat::Q0)?; // A
        }
    }

    // 2. Testiranje Overwrite-a na poziciji 1 (menjamo drugi 'A' u 'C')
    {
        let mut file = file_handle.lock().unwrap();
        file.seek_quat(1)?;
        file.write_quat(Quat::Q1)?; // Menjamo u C
    }

    // 3. Provera rezultata
    {
        let mut file = file_handle.lock().unwrap();
        file.seek_quat(0)?;

        print!("The result after the change in the middle (expected ACGA/ACAA): ");
        while let Some(q) = file.read_quat()? {
            print!("{}", q.as_char());
        }
        println!();
    }

    // 4. Testiranje Brisanja
    vfs.delete("/test.quat")?;
    println!("File successfully deleted from VFS.");

    Ok(())
}