pub mod buffer;
pub mod quat;
pub mod traits;
pub mod vfs;
pub mod virtual_file;
pub mod compressed_file;
pub mod error;
pub mod ffi;

// Re-exportovanje osnovnih tipova za lakši uvoz
pub use buffer::QuatBuffer;
pub use quat::Quat;
pub use traits::{QuatFile, QuatFileSystem};
pub use vfs::{FileHandle, MemoryQuatVFS};
pub use virtual_file::VirtualQuatFile;
pub use compressed_file::CompressedQuatFile;
use crate::ffi::*;
pub use std::os::raw::c_int;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::QuatBuffer;
    use crate::compressed_file::CompressedQuatFile;
    use crate::error::{QuatError, Result};
    use crate::quat::Quat;
    use crate::traits::{QuatFile, QuatFileSystem};
    use crate::vfs::MemoryQuatVFS;
    use std::sync::Arc;
    use std::ffi::CString;
    use std::ptr;
    use std::thread;

    #[test]
    fn test_quat_conversions() -> Result<()> {
        assert_eq!(Quat::try_from('A')?, Quat::Q0);
        assert_eq!(Quat::try_from('C')?, Quat::Q1);
        assert_eq!(Quat::try_from('G')?, Quat::Q2);
        assert_eq!(Quat::try_from('T')?, Quat::Q3);

        assert!(Quat::try_from('X').is_err());
        Ok(())
    }

    #[test]
    fn test_vfs_overwrite_and_delete() -> Result<()> {
        let mut vfs = MemoryQuatVFS::new();
        let mut file_handle = vfs.create("/test.quat")?;

        for _ in 0..4 {
            file_handle.write_quat(Quat::Q0)?; // AAAA
        }

        file_handle.seek_quat(1)?;
        file_handle.write_quat(Quat::Q1)?; // ACAA

        file_handle.seek_quat(0)?;
        let mut read_chars = String::new();
        while let Some(q) = file_handle.read_quat()? {
            read_chars.push(q.as_char());
        }

        assert_eq!(read_chars, "ACAA");

        vfs.delete("/test.quat")?;
        assert!(vfs.open("/test.quat").is_err());
        Ok(())
    }

    #[test]
    fn test_vfs_seek_past_eof() -> Result<()> {
        let mut vfs = MemoryQuatVFS::new();
        let mut file_handle = vfs.create("/seek_test.quat")?;

        file_handle.write_quat(Quat::Q3)?; // T na indeksu 0
        file_handle.seek_quat(4)?;          // Pomak na indeks 4 (ostavlja rupu)
        file_handle.write_quat(Quat::Q2)?; // G na indeksu 4

        file_handle.seek_quat(0)?;
        let mut buf = [Quat::Q0; 5];
        let read_count = file_handle.read_exact_quats(&mut buf)?;

        assert_eq!(read_count, 5);
        // Očekujemo T, A, A, A, G jer su neupisana mjesta popunjena s Q0 (A)
        assert_eq!(buf, [Quat::Q3, Quat::Q0, Quat::Q0, Quat::Q0, Quat::Q2]);
        Ok(())
    }

    #[test]
    fn test_vfs_disk_io_and_magic_header() -> Result<()> {
        let mut vfs = MemoryQuatVFS::new();
        let mut handle = vfs.create("/dna.quat")?;

        handle.write_quat(Quat::Q0)?; // A
        handle.write_quat(Quat::Q1)?; // C
        handle.write_quat(Quat::Q2)?; // G
        handle.write_quat(Quat::Q3)?; // T

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("temp_dna.bin");
        let temp_path_str = temp_path.to_str().unwrap();

        vfs.save_to_disk("/dna.quat", temp_path_str)?;

        let loaded_handle = vfs.load_from_disk(temp_path_str, "/loaded_dna.quat")?;
        let mut read_buf = [Quat::Q0; 4];
        let mut loaded_file = loaded_handle;

        let count = loaded_file.read_exact_quats(&mut read_buf)?;
        assert_eq!(count, 4);
        assert_eq!(read_buf, [Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3]);

        let _ = std::fs::remove_file(temp_path);
        Ok(())
    }

    #[test]
    fn test_vfs_stress_large_data() -> Result<()> {
        let mut vfs = MemoryQuatVFS::new();
        let mut handle = vfs.create("/stress.quat")?;
        let total_quats = 100_000;

        for i in 0..total_quats {
            let q = Quat::from_u8_masked((i % 4) as u8);
            handle.write_quat(q)?;
        }

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("temp_stress.bin");
        let temp_path_str = temp_path.to_str().unwrap();

        vfs.save_to_disk("/stress.quat", temp_path_str)?;

        let mut loaded_handle = vfs.load_from_disk(temp_path_str, "/loaded_stress.quat")?;
        assert_eq!(loaded_handle.len(), total_quats);

        loaded_handle.seek_quat(0)?;
        for i in 0..total_quats {
            let expected = Quat::from_u8_masked((i % 4) as u8);
            let read = loaded_handle.read_quat()?.unwrap();
            assert_eq!(read, expected, "Mismatch at index {}", i);
        }

        let _ = std::fs::remove_file(temp_path);
        Ok(())
    }

    #[test]
    fn test_vfs_concurrent_thread_safety() -> Result<()> {
        let vfs = Arc::new(std::sync::Mutex::new(MemoryQuatVFS::new()));
        let file_handle = vfs.lock().unwrap().create("/concurrent.quat")?;

        let mut handles = vec![];

        for i in 0..8 {
            let mut handle_clone = file_handle.clone();
            let handle = thread::spawn(move || {
                let quat = Quat::from_u8_masked((i % 4) as u8);
                handle_clone.write_quat(quat)
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()?;
        }

        assert_eq!(file_handle.len(), 8);
        Ok(())
    }

    #[test]
fn test_vfs_corrupted_disk_header() {
    let temp_dir = std::env::temp_dir();
    let bad_header_path = temp_dir.join("corrupted_header.bin");

    // Zapisujemo 12 neispravnih bajtova (ne počinju sa "QVFS")
    let bad_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    std::fs::write(&bad_header_path, bad_bytes).unwrap();

    let vfs = MemoryQuatVFS::new();
    let result = vfs.load_from_disk(bad_header_path.to_str().unwrap(), "/bad.quat");

    assert!(result.is_err());
    assert!(matches!(result, Err(QuatError::InvalidHeader)));

    let _ = std::fs::remove_file(bad_header_path);
}

    #[test]
    fn test_compressed_quat_file_read_only() -> Result<()> {
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q2);
        buf.push(Quat::Q3);

        let mut compressed_file = CompressedQuatFile::new(buf);

        assert_eq!(compressed_file.read_quat()?, Some(Quat::Q2));
        assert_eq!(compressed_file.read_quat()?, Some(Quat::Q3));
        assert_eq!(compressed_file.read_quat()?, None);

        let write_res = compressed_file.write_quat(Quat::Q0);
        assert!(write_res.is_err());
        assert!(matches!(write_res, Err(QuatError::PermissionDenied(_))));

        Ok(())
    }
    #[test]
    fn test_ffi_vfs_lifecycle() {
        unsafe {
            let vfs = qvfs_vfs_new();
            assert!(!vfs.is_null());

            let path = CString::new("test.bin").unwrap();
            let mut handle: *mut FileHandle = ptr::null_mut();

            // Kreiranje fajla
            let res = qvfs_vfs_create(vfs, path.as_ptr(), &mut handle);
            assert_eq!(res, 0);
            assert!(!handle.is_null());

            // Pisanje kvata (0=Q0, 1=Q1, 2=Q2, 3=Q3)
            assert_eq!(qvfs_file_write_quat(handle, 0), 0);
            assert_eq!(qvfs_file_write_quat(handle, 3), 0);

            // Provera dužine
            let mut len: usize = 0;
            assert_eq!(qvfs_file_len(handle, &mut len), 0);
            assert_eq!(len, 2);

            // Seek na početak
            let mut new_pos: usize = 0;
            assert_eq!(qvfs_file_seek(handle, 0, &mut new_pos), 0);
            assert_eq!(new_pos, 0);

            // Čitanje prvog kvata
            let mut val: u8 = 255;
            let mut has_val: c_int = 0;
            assert_eq!(qvfs_file_read_quat(handle, &mut val, &mut has_val), 0);
            assert_eq!(has_val, 1);
            assert_eq!(val, 0);

            // Čitanje drugog kvata
            assert_eq!(qvfs_file_read_quat(handle, &mut val, &mut has_val), 0);
            assert_eq!(has_val, 1);
            assert_eq!(val, 3);

            // Oslobađanje memorije
            qvfs_file_free(handle);
            qvfs_vfs_free(vfs);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            // Svi pozivi sa NULL pokazivačima moraju vratiti -1 umesto da sruše aplikaciju
            assert_eq!(qvfs_vfs_create(ptr::null_mut(), ptr::null(), ptr::null_mut()), -1);
            assert_eq!(qvfs_vfs_open(ptr::null_mut(), ptr::null(), ptr::null_mut()), -1);
            assert_eq!(qvfs_vfs_delete(ptr::null_mut(), ptr::null()), -1);
            assert_eq!(qvfs_file_write_quat(ptr::null_mut(), 0), -1);
            assert_eq!(qvfs_file_write_quat(ptr::null_mut(), 99), -1); // Nevalidan kvat (>3)
            assert_eq!(qvfs_file_read_quat(ptr::null_mut(), ptr::null_mut(), ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_ffi_batch_read() {
        unsafe {
            let vfs = qvfs_vfs_new();
            let path = CString::new("batch.bin").unwrap();
            let mut handle: *mut FileHandle = ptr::null_mut();

            qvfs_vfs_create(vfs, path.as_ptr(), &mut handle);

            // Upis 4 kvata (Q0, Q1, Q2, Q3)
            for q in 0..4 {
                qvfs_file_write_quat(handle, q);
            }

            // Seek na početak
            let mut new_pos: usize = 0;
            qvfs_file_seek(handle, 0, &mut new_pos);

            // Čitanje u bafer
            let mut buf = [0u8; 4];
            let mut read_count: usize = 0;
            let res = qvfs_file_read_exact_quats(handle, buf.as_mut_ptr(), 4, &mut read_count);

            assert_eq!(res, 0);
            assert_eq!(read_count, 4);
            assert_eq!(buf, [0, 1, 2, 3]);

            qvfs_file_free(handle);
            qvfs_vfs_free(vfs);
        }
    }
    
}
