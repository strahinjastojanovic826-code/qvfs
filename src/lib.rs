pub mod buffer;
pub mod quat;
pub mod traits;
pub mod vfs;
pub mod virtual_file;
pub mod compressed_file;
pub mod error;
pub mod ffi;
pub mod dna;
pub mod async_vfs;
pub mod mmap_vfs;
pub mod wal;
pub mod container;

// Re-exportovanje osnovnih tipova za lakši uvoz
pub use buffer::QuatBuffer;
pub use quat::Quat;
pub use traits::{QuatFile, QuatFileSystem};
pub use vfs::{FileHandle, MemoryQuatVFS};
pub use virtual_file::VirtualQuatFile;
pub use compressed_file::CompressedQuatFile;
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
    use std::path::PathBuf;
    use std::sync::Mutex;
    use crate::ffi::qvfs_vfs_new;
    use crate::ffi::qvfs_vfs_create;
    use crate::ffi::qvfs_file_write_quat;
    use crate::ffi::qvfs_file_len;
    use crate::ffi::qvfs_file_seek;
    use crate::ffi::qvfs_file_read_quat;
    use crate::ffi::qvfs_file_free;
    use crate::ffi::qvfs_vfs_open;
    use crate::ffi::qvfs_vfs_delete;
    use crate::ffi::qvfs_vfs_free;
    use crate::ffi::qvfs_file_read_exact_quats;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
fn test_quat_conversions() -> Result<()> {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    assert_eq!(Quat::try_from('A')?, Quat::Q0);
    assert_eq!(Quat::try_from('C')?, Quat::Q1);
    assert_eq!(Quat::try_from('G')?, Quat::Q2);
    assert_eq!(Quat::try_from('T')?, Quat::Q3);

    assert!(Quat::try_from('X').is_err());

    let duration = start.elapsed();
    println!("[BENCHMARK] test_quat_conversions: Executed character-to-quat mappings in {:?}", duration);
    // --- BENCHMARK END ---

    Ok(())
}

    #[test]
fn test_vfs_overwrite_and_delete() -> Result<()> {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut vfs = MemoryQuatVFS::new();
    let mut file_handle: Arc<Mutex<VirtualQuatFile>> = vfs.create("/test.quat")?;

    for _ in 0..4 {
        file_handle.write_quat(Quat::Q0)?; // AAAA
    }

    file_handle.seek_quat(1)?;
    file_handle.write_quat(Quat::Q1)?; // ACAA

    file_handle.seek_quat(0)?;
    let mut read_chars: String = String::new();
    while let Some(q) = file_handle.read_quat()? {
        read_chars.push(q.as_char());
    }

    assert_eq!(read_chars, "ACAA");

    vfs.delete("/test.quat")?;
    assert!(vfs.open("/test.quat").is_err());

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_overwrite_and_delete: VFS creation, write, seek, overwrite, read & delete in {:?}", duration);
    // --- BENCHMARK END ---

    Ok(())
}

    #[test]
fn test_vfs_seek_past_eof() -> Result<()> {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut vfs = MemoryQuatVFS::new();
    let mut file_handle = vfs.create("/seek_test.quat")?;

    file_handle.write_quat(Quat::Q3)?; // T na indeksu 0
    file_handle.seek_quat(4)?; // Pomak na indeks 4 (ostavlja rupu)
    file_handle.write_quat(Quat::Q2)?; // G na indeksu 4

    file_handle.seek_quat(0)?;
    let mut buf = [Quat::Q0; 5];
    let read_count: usize = file_handle.read_exact_quats(&mut buf)?;

    assert_eq!(read_count, 5);
    assert_eq!(buf, [Quat::Q3, Quat::Q0, Quat::Q0, Quat::Q0, Quat::Q2]);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_seek_past_eof: Seek past EOF padding & exact read in {:?}", duration);
    // --- BENCHMARK END ---

    Ok(())
}

    #[test]
fn test_vfs_disk_io_and_magic_header() -> Result<()> {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut vfs = MemoryQuatVFS::new();
    let mut handle = vfs.create("/dna.quat")?;

    handle.write_quat(Quat::Q0)?; // A
    handle.write_quat(Quat::Q1)?; // C
    handle.write_quat(Quat::Q2)?; // G
    handle.write_quat(Quat::Q3)?; // T

    let temp_dir: PathBuf = std::env::temp_dir();
    let temp_path: PathBuf = temp_dir.join("temp_dna.bin");
    let temp_path_str: &str = &temp_path.to_str().unwrap();

    vfs.save_to_disk("/dna.quat", temp_path_str)?;

    let mut loaded_handle = vfs.load_from_disk(temp_path_str, "/loaded_dna.quat")?;
    let mut read_buf: [Quat; 4] = [Quat::Q0; 4];
    let count: usize = loaded_handle.read_exact_quats(&mut read_buf)?;
    assert_eq!(count, 4);
    assert_eq!(read_buf, [Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3]);

    let _ = std::fs::remove_file(temp_path);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_disk_io_and_magic_header: Disk save, load, header & exact quat read in {:?}", duration);
    // --- BENCHMARK END ---

    Ok(())
}

      #[test]
fn test_vfs_stress_large_data() -> Result<()> {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut vfs = MemoryQuatVFS::new();
    let mut handle = vfs.create("/stress.quat")?;
    let total_quats: usize = 100_000;

    for i in 0..total_quats {
        let q: Quat = Quat::from_u8_masked((i % 4) as u8);
        handle.write_quat(q)?;
    }

    let temp_dir: PathBuf = std::env::temp_dir();
    let temp_path: PathBuf = temp_dir.join("temp_stress.bin");
    let temp_path_str: &str = &temp_path.to_str().unwrap();

    vfs.save_to_disk("/stress.quat", temp_path_str)?;

    let mut loaded_handle = vfs.load_from_disk(temp_path_str, "/loaded_stress.quat")?;
    assert_eq!(loaded_handle.len(), total_quats);

    loaded_handle.seek_quat(0)?;
    for i in 0..total_quats {
        let expected: Quat = Quat::from_u8_masked((i % 4) as u8);
        let read: Quat = loaded_handle.read_quat()?.unwrap();
        assert_eq!(read, expected, "Mismatch at index {}", i);
    }

    let _ = std::fs::remove_file(temp_path);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_stress_large_data: 100,000 quats written, saved, loaded & verified in {:?}", duration);
    // --- BENCHMARK END ---

    Ok(())
}

     #[test]
fn test_vfs_concurrent_thread_safety() -> Result<()> {
    let start = std::time::Instant::now();

    let vfs = Arc::new(Mutex::new(MemoryQuatVFS::new()));
    let file_handle = vfs.lock().unwrap().create("/concurrent.quat")?;

    let mut handles = vec![];

    for i in 0..8 {
        let mut handle_clone = file_handle.clone();
        let handle = thread::spawn(move || {
            let q = Quat::from_u8_masked((i % 4) as u8);
            handle_clone.write_quat(q)
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    assert_eq!(file_handle.len(), 8);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_concurrent_thread_safety: Multi-threaded concurrent writes across 8 threads in {:?}", duration);

    Ok(())
}

    #[test]
fn test_vfs_corrupted_disk_header() -> Result<()> {
    let start = std::time::Instant::now();

    let temp_dir = std::env::temp_dir();
    let bad_header_path = temp_dir.join("corrupted_header.bin");

    let bad_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    std::fs::write(&bad_header_path, bad_bytes)?;

    let vfs = MemoryQuatVFS::new();
    let result = vfs.load_from_disk(bad_header_path.to_str().unwrap(), "/bad.quat");

    assert!(result.is_err());
    assert!(matches!(result, Err(QuatError::InvalidHeader)));
    let _ = std::fs::remove_file(bad_header_path);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_corrupted_header: Error handling validation for corrupted binary headers in {:?}", duration);
    Ok(())
}

    #[test]
fn test_compressed_quat_file_read_only() -> Result<()> {
    let start = std::time::Instant::now();

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

    let duration = start.elapsed();
    println!("[BENCHMARK] test_compressed_quat_file_read_only: Read-only buffer validation in {:?}", duration);
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
    
    #[test]
fn test_quaternary_logic_operations() {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    assert_eq!(Quat::Q0.q_not(), Quat::Q3);
    assert_eq!(Quat::Q1.q_not(), Quat::Q2);

    assert_eq!(Quat::Q0.q_and(Quat::Q3), Quat::Q0);
    assert_eq!(Quat::Q1.q_or(Quat::Q2), Quat::Q3);
    assert_eq!(Quat::Q2.q_xor(Quat::Q2), Quat::Q0);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_quaternary_logic_operations: Quat bitwise logical calculations in {:?}", duration);
    // --- BENCHMARK END ---
}

      #[test]
fn test_vfs_transform_file_logical() -> Result<()> {
    let start = std::time::Instant::now();

    let mut vfs = MemoryQuatVFS::new();
    let handle = vfs.create("/logic.quat")?;

    {
        let mut file = handle.lock().unwrap();
        file.write_quat(Quat::Q0)?;
        file.write_quat(Quat::Q1)?;
    }

    vfs.transform_file_logical("/logic.quat", |q| q.q_not())?;

    let mut file = handle.lock().unwrap();
    file.seek_quat(0)?;
    assert_eq!(file.read_quat()?, Some(Quat::Q3));
    assert_eq!(file.read_quat()?, Some(Quat::Q2));

    let duration = start.elapsed();
    println!("[BENCHMARK] test_vfs_transform_file_logical: In-place file VFS logic transformations in {:?}", duration);
    Ok(())
}

#[test]
fn test_quat_buffer_iterators() {
    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut buf: QuatBuffer = QuatBuffer::new();
    buf.push(Quat::Q0);
    buf.push(Quat::Q1);
    buf.push(Quat::Q2);

    let collected: Vec<Quat> = buf.iter().collect();
    assert_eq!(collected, vec![Quat::Q0, Quat::Q1, Quat::Q2]);

    let duration = start.elapsed();
    println!("[BENCHMARK] test_quat_buffer_iterators: QuatBuffer iteration traversal in {:?}", duration);
    // --- BENCHMARK END ---
}

#[test]
fn test_assembly_ffi_packing_performance() {
    let src_data: Vec<u8> = vec![0, 1, 2, 3, 0, 1, 2, 3]; // vrednosti kvarta (0-3)
    let mut dest_data: Vec<u8> = vec![0; 4];

    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    // Ponavljamo 100,000 puta da dobijemo merljivo vreme
    for _ in 0..100_000 {
        unsafe {
            let res = crate::ffi::qvfs_asm_pack_buffer(
                src_data.as_ptr(),
                src_data.len(),
                dest_data.as_mut_ptr(),
                dest_data.len(),
            );
            assert_eq!(res, 0);
        }
    }

    let duration = start.elapsed();
    println!("[BENCHMARK] ASM FFI Pack: 100,000 iterations executed in {:?}", duration);
    // --- BENCHMARK END ---
}

#[tokio::test]
async fn test_async_vfs_throughput() -> Result<()> {
    use crate::async_vfs::AsyncMemoryQuatVFS;
    use crate::buffer::QuatBuffer;
    use crate::quat::Quat;
    use std::fs;
    use std::time::{Instant, Duration};

    let storage_path = "async_test.bin";
    let _ = fs::remove_file(&storage_path);

    let vfs = AsyncMemoryQuatVFS::new(storage_path);
    let mut buf = QuatBuffer::new();
    for _ in 0..5000 {
        buf.push(Quat::Q1);
    }

    // --- BENCHMARK START ---
    let start = Instant::now();

    // Ručno pripremamo fajl u tačnom formatu koji load_from_disk_async očekuje (16 bajtova zaglavlja)
    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(b"QVFS");                  // 4 bajta: magični header (0..4)
    file_bytes.extend_from_slice(&[0u8; 4]);                // 4 bajta: placeholder za checksum (4..8)
    file_bytes.extend_from_slice(&(5000u64.to_le_bytes()));   // 8 bajtova: len_quats na opsegu (8..16)
    file_bytes.extend_from_slice(&buf.data);                // Podaci od indeksa 16 nadalje
    
    fs::write(&storage_path, file_bytes).unwrap();

    // Asinhrono učitavanje
    let loaded_buf = vfs.load_from_disk_async().await?;

    let duration: Duration = start.elapsed();
    println!("[BENCHMARK] Tokio Async VFS: Save & Load 5,000 quats in {:?}", duration);
    // --- BENCHMARK END ---

    assert_eq!(loaded_buf.len_quats, 5000);
    let _ = fs::remove_file(storage_path);
    Ok(())
}

#[test]
fn test_mmap_vfs_speed_and_safety() -> Result<()> {
    use crate::mmap_vfs::MmapQuatFile;
    use crate::quat::Quat;
    use std::fs;

    let mmap_path = "test_mmap.bin";
    let _ = fs::remove_file(&mmap_path);

    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut mmap_file = MmapQuatFile::new(&mmap_path)?;
    
    // Upisujemo 10,000 kvarta direktno u memorijski mapiran fajl
    for i in 0..10_000 {
        mmap_file.set_quat(i, Quat::Q3);
    }

    // Čitamo nazad
    for i in 0..10_000 {
        let q = mmap_file.get_quat(i);
        assert_eq!(q, Some(Quat::Q3));
    }

    let duration = start.elapsed();
    println!("[BENCHMARK] MmapQuatFile: 10,000 random read/writes completed in {:?}", duration);
    // --- BENCHMARK END ---

    let _ = fs::remove_file(mmap_path);
    Ok(())
}

#[test]
fn test_wal_transaction_logging_benchmark() -> Result<()> {
    use crate::wal::WriteAheadLog;
    use crate::quat::Quat;
    use std::fs;

    let wal_path = "test_trans.wal";
    let _ = fs::remove_file(&wal_path);

    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut wal = WriteAheadLog::new(&wal_path)?;
    
    // Simuliramo 1000 transakcionih upisa kvartarnih podataka
    for i in 0..1000 {
        wal.log_write(i, Quat::Q2)?;
    }
    wal.commit()?;

    let duration = start.elapsed();
    println!("[BENCHMARK] WAL: 1,000 transaction logs written & committed in {:?}", duration);
    // --- BENCHMARK END ---

    assert!(std::path::Path::new(wal_path).exists());
    let _ = fs::remove_file(wal_path);
    Ok(())
}

#[test]
fn test_container_performance_and_validity() -> Result<()> {
    use crate::container::QuatContainer;
    use crate::buffer::QuatBuffer;
    use crate::quat::Quat;
    use std::fs;

    let container_path = "test_vfs_container.qcnt";
    let _ = fs::remove_file(&container_path);

    // --- BENCHMARK START ---
    let start = std::time::Instant::now();

    let mut container = QuatContainer::open(&container_path)?;
    
    // Upisujemo 100 različitih bafera u jedan kontejner
    for i in 0..100 {
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q0);
        buf.push(Quat::Q1);
        let path = format!("/file_{}.quat", i);
        container.write_file_buffer(&path, &buf)?;
    }

    let duration = start.elapsed();
    println!("[BENCHMARK] QuatContainer: 100 files packed & written in {:?}", duration);
    // --- BENCHMARK END ---

    assert!(std::path::Path::new(container_path).exists());
    let _ = fs::remove_file(container_path);
    Ok(())
}

  #[test]
    fn test_quat_mod4_and_complement_ops() -> Result<()> {
        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        assert_eq!(Quat::Q0.complement(), Quat::Q3);
        assert_eq!(Quat::Q1.complement(), Quat::Q2);
        assert_eq!(Quat::Q0.q_add_mod4(Quat::Q3), Quat::Q3);
        assert_eq!(Quat::Q2.q_add_mod4(Quat::Q2), Quat::Q0);
        assert_eq!(Quat::Q0.q_sub_mod4(Quat::Q1), Quat::Q3);
        assert_eq!(Quat::Q0.q_inc(), Quat::Q1);
        assert_eq!(Quat::Q0.q_dec(), Quat::Q3);

        let packed = Quat::pack_4([Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3]);
        assert_eq!(packed, 0b00_01_10_11);
        assert_eq!(
            Quat::unpack_4(packed),
            [Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3]
        );

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_quat_mod4_and_complement_ops: Quaternary mod-4 & pack/unpack in {:?}",
            duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

    #[test]
    fn test_buffer_batch_swar_and_hamming() -> Result<()> {
        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        let total = 10_000usize;
        let input_quats: Vec<Quat> = (0..total)
            .map(|i| Quat::from_u8_masked((i % 4) as u8))
            .collect();

        let mut buf_a = QuatBuffer::new();
        buf_a.extend_from_quats(&input_quats);
        assert_eq!(buf_a.len_quats, total);

        let unpacked = buf_a.to_quat_vec();
        assert_eq!(unpacked, input_quats);

        let mut slice_out = vec![Quat::Q0; 128];
        let read_n = buf_a.unpack_into_slice(17, &mut slice_out);
        assert_eq!(read_n, 128);
        assert_eq!(slice_out, &input_quats[17..17 + 128]);

        let counts = buf_a.count_states();
        assert!(counts[0] >= 2500);
        assert_eq!(counts[1], 2500);
        assert_eq!(counts[2], 2500);
        assert_eq!(counts[3], 2500);

        let mut buf_b = QuatBuffer::new();
        buf_b.extend_from_quats(&vec![Quat::Q0; total]);
        assert_eq!(buf_a.hamming_distance(&buf_b), Some(7500));

        let sum_buf = buf_a.batch_add_mod4(&buf_b).unwrap();
        assert_eq!(sum_buf.to_quat_vec(), input_quats);

        let mut comp_buf = buf_a.clone();
        comp_buf.complement_in_place();
        assert_eq!(comp_buf.get(0), Some(Quat::Q3));

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_buffer_batch_swar_and_hamming: Batch & SWAR ops on {} quats in {:?}",
            total, duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

    #[test]
    fn test_dna_fast_batch_conversion() -> Result<()> {
        use crate::dna::DnaStorage;

        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        let dna_seq = "ACGTACGTACGTACGTACGTACGTACGTACGT";
        let buf = DnaStorage::bytes_to_buffer(dna_seq.as_bytes())?;
        assert_eq!(buf.len_quats, dna_seq.len());

        let roundtrip = DnaStorage::buffer_to_dna_fast(&buf);
        assert_eq!(roundtrip, dna_seq);

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_dna_fast_batch_conversion: Fast DNA encode/decode in {:?}",
            duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

    #[test]
    fn test_versioned_frame_and_rle_compression() -> Result<()> {
        use crate::compressed_file::CompressedQuatFile;
        use crate::vfs::{decode_versioned_frame, encode_versioned_frame, QVFS_FLAG_NONE};

        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        let mut buf = QuatBuffer::new();
        buf.extend_from_quats(&vec![Quat::Q2; 64]);
        buf.extend_from_quats(&vec![Quat::Q1; 36]);

        let encoded = encode_versioned_frame(&buf.data, buf.len_quats as u64, QVFS_FLAG_NONE)?;
        let (flags, total_quats, payload) = decode_versioned_frame(&encoded)?;
        assert_eq!(flags, QVFS_FLAG_NONE);
        assert_eq!(total_quats, 100);
        assert_eq!(payload, buf.data);

        let segments = CompressedQuatFile::segment_quats(&buf, 32);
        assert_eq!(segments.len(), 4);

        let rle = CompressedQuatFile::compress_rle_quats(&buf);
        let decompressed = CompressedQuatFile::decompress_rle_quats(&rle, buf.len_quats)?;
        assert_eq!(decompressed, buf);

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_versioned_frame_and_rle_compression: QVFH v1, segment & RLE in {:?}",
            duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

    #[test]
    fn test_wal_replay_and_commit_truncate() -> Result<()> {
        use crate::wal::WriteAheadLog;
        use std::fs;
        use std::path::PathBuf;

        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        let wal_path: PathBuf = std::env::temp_dir().join("qvfs_test_replay_fixed.wal");
        let _ = fs::remove_file(&wal_path);
        let wal_str = wal_path.to_str().unwrap();

        {
            let mut wal = WriteAheadLog::new(wal_str)?;
            wal.log_begin()?;
            wal.log_write(0, Quat::Q1)?;
            wal.log_write(1, Quat::Q2)?;
            wal.commit()?;

            wal.log_begin()?;
            wal.log_write(2, Quat::Q3)?;
        }

        let mut target = QuatBuffer::new();
        {
            let mut wal = WriteAheadLog::new(wal_str)?;
            let applied = wal.replay_committed(&mut target)?;
            assert_eq!(applied, 2);
            assert_eq!(target.get(0), Some(Quat::Q1));
            assert_eq!(target.get(1), Some(Quat::Q2));
            assert_eq!(target.get(2), None);

            wal.commit()?;
        }

        assert!(std::path::Path::new(&wal_path).exists());
        let _ = fs::remove_file(&wal_path);

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_wal_replay_and_commit_truncate: WAL replay & commit in {:?}",
            duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

    #[test]
    fn test_ffi_strict_write_and_exists() -> Result<()> {
        use crate::ffi::{
            qvfs_file_free, qvfs_file_read_quat, qvfs_file_seek, qvfs_file_write_quat_strict,
            qvfs_vfs_create, qvfs_vfs_exists, qvfs_vfs_free, qvfs_vfs_new, QvfsFfiStatus,
        };
        use crate::FileHandle;
        use std::ffi::CString;
        use std::os::raw::c_int;
        use std::ptr;

        // --- BENCHMARK START ---
        let start: Instant = std::time::Instant::now();

        unsafe {
            let vfs = qvfs_vfs_new();
            assert!(!vfs.is_null());

            let path = CString::new("/strict.quat").unwrap();
            let mut handle: *mut FileHandle = ptr::null_mut();
            assert_eq!(qvfs_vfs_create(vfs, path.as_ptr(), &mut handle), 0);

            let mut exists: c_int = 0;
            assert_eq!(
                qvfs_vfs_exists(vfs, path.as_ptr(), &mut exists),
                QvfsFfiStatus::Ok as c_int
            );
            assert_eq!(exists, 1);

            assert_eq!(
                qvfs_file_write_quat_strict(handle, 2),
                QvfsFfiStatus::Ok as c_int
            );
            assert_eq!(
                qvfs_file_write_quat_strict(handle, 9),
                QvfsFfiStatus::ErrInvalidQuatValue as c_int
            );

            let mut new_pos = 0usize;
            assert_eq!(qvfs_file_seek(handle, 0, &mut new_pos), 0);

            let mut q_val = 0u8;
            let mut has_val: c_int = 0;
            assert_eq!(
                qvfs_file_read_quat(handle, &mut q_val, &mut has_val),
                0
            );
            assert_eq!(has_val, 1);
            assert_eq!(q_val, 2);

            qvfs_file_free(handle);
            qvfs_vfs_free(vfs);
        }

        let duration: Duration = start.elapsed();
        println!(
            "[BENCHMARK] test_ffi_strict_write_and_exists: FFI strict validation in {:?}",
            duration
        );
        // --- BENCHMARK END ---

        Ok(())
    }

}
