use crate::error::Result;
use crate::quat::Quat;

pub trait QuatFile {
    fn read_quat(&mut self) -> Result<Option<Quat>>;
    fn write_quat(&mut self, quat: Quat) -> Result<()>;
    fn seek_quat(&mut self, quat_offset: usize) -> Result<usize>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn read_exact_quats(&mut self, buf: &mut [Quat]) -> Result<usize>;
}

pub trait QuatFileSystem {
    type FileHandle: QuatFile;

    fn create(&mut self, path: &str) -> Result<Self::FileHandle>;
    fn open(&mut self, path: &str) -> Result<Self::FileHandle>;
    fn delete(&mut self, path: &str) -> Result<()>;
}