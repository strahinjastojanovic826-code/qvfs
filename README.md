# qvfs

`qvfs` is a Rust-based Quaternary Virtual File System (QVFS) simulator. It models file system operations using 4-state (quaternary) logic representation (Q0, Q1, Q2, Q3) mapped to 2-bit quantum/biological states (e.g., Adenine, Cytosine, Guanine, Thymine).

## Overview

Unlike standard binary file systems operating strictly on bits (0 and 1), `qvfs` implements storage and state manipulation on 2-bit quats. This crate provides data structures and a virtual execution context for experimenting with multi-state logic systems.

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `qvfs` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

## Features

* **Quat Primitive Types:** Core 4-state primitives (Q0, Q1, Q2, Q3) with bitwise conversion utilities.
* **Biomorphic Mapping:** Built-in representation of DNA bases (A, C, G, T) mapped directly to 2-bit boundaries.
* **In-Memory VFS:** Thread-safe virtual file system (`MemoryQuatVFS`) supporting file creation, modification, deletion, and directory listings (`list_dir`).
* **Binary Persistence with Magic Header:** Custom file format with a mandatory 4-byte `QVFS` header and high-performance serialization (`save_to_disk` / `load_from_disk`).
* **Compressed File Extensions:** Support for read-only compressed quat structures via `CompressedQuatFile` and buffer-based search operations (`QuatBuffer`).
* **Custom Error Handling:** Strongly-typed `QuatError` enum for exact failure diagnosis (header corruption, invalid characters, permission issues).
* **Quaternary Logic Gates:** Built-in multi-valued logic operations (`q_not`, `q_and`, `q_or`, `q_xor`) for quaternary states.
* **Standard Rust Iterators:** Full `Iterator` and `IntoIterator` support for `QuatBuffer` allowing seamless functional programming (`.map()`, `.filter()`, etc.).
* **Data Integrity & Checksum:** Built-in FNV-1a checksum validation embedded in disk headers to protect against data corruption.
* **VFS Logical Transformations:** In-place logical transformations on virtual files via `transform_file_logical`.
* **Single-File VFS Containers:** Packing multiple virtual files into a unified binary container format (`container.rs`)[cite: 20].
* **Write-Ahead Logging (WAL):** Transactional logging and crash-safe recovery system for data integrity (`wal.rs`)[cite: 20].
* **Memory-Mapping (mmap):** Direct memory-mapped file access using `memmap2` for ultra-high performance quaternary operations (`mmap_vfs.rs`)[cite: 20].
* **Asynchronous VFS Support:** Non-blocking asynchronous file saving and loading powered by Tokio (`async_vfs.rs`)[cite: 20].
* **Assembly-Friendly FFI:** Flat, raw-pointer `extern "C"` functions optimized for direct integration with Assembly and low-level systems (`ffi.rs`)[cite: 20].
* **Quaternary-to-Symbolic Mapping:** Flexible domain mapping for 4-state quaternary data (`Q0`, `Q1`, `Q2`, `Q3`)[cite: 20].
* **Batch & SWAR Buffer Operations:** Vectorized packing/unpacking, state counting, Hamming distance, modulo-4 arithmetic, and in-place complement on `QuatBuffer`.
* **Versioned `QVFH` v1 Framing & RLE:** Header-versioned binary frame encoding/decoding alongside quaternary segmentation and run-length compression/decompression.
* **WAL Replay & Truncation Controls:** Committed-transaction replay (`replay_committed`) and log truncation (`commit_and_truncate`) to isolate uncommitted writes.
* **Strict & Batch C FFI:** Explicit `QvfsFfiStatus` error codes, strict quaternary input validation (`qvfs_file_write_quat_strict`), batch writes (`qvfs_file_write_quats_batch`), and path existence checks (`qvfs_vfs_exists`).

## Benchmark tests

| Test / Benchmark Name | Workload / Scenario | Measured Time |
| :--- | :--- | :--- |
| `test_quaternary_logic_operations` | `Quat` bitwise logical calculations[cite: 25, 26] | `400ns`[cite: 25, 26] |
| `test_quat_mod4_and_complement_ops` | Quaternary mod-4 & pack/unpack operations[cite: 25, 26] | `700ns`[cite: 25, 26] |
| `test_quat_conversions` | Character-to-quat mappings[cite: 25, 26] | `1.7µs`[cite: 25] |
| `test_quat_buffer_iterators` | `QuatBuffer` iteration traversal[cite: 25] | `2.3µs`[cite: 25] |
| `test_dna_fast_batch_conversion` | Fast DNA encode/decode[cite: 25, 26] | `10.7µs`[cite: 25] |
| `test_vfs_seek_past_eof` | Seek past EOF padding & exact read[cite: 25, 26] | `14.7µs`[cite: 25] |
| `test_ffi_strict_write_and_exists` | FFI strict validation (`qvfs_file_write_quat_strict` & `qvfs_vfs_exists`)[cite: 25] | `17.3µs`[cite: 25] |
| `test_vfs_transform_file_logical` | In-place file VFS logic transformations[cite: 26] | `19µs`[cite: 26] |
| `test_compressed_quat_file_read_only` | Read-only buffer validation[cite: 25] | `19.3µs`[cite: 25] |
| `test_vfs_overwrite_and_delete` | VFS creation, write, seek, overwrite, read & delete[cite: 25, 26] | `20µs`[cite: 25] |
| `test_versioned_frame_and_rle_compression` | `QVFH` v1 framing, segmentation & RLE compression/decompression[cite: 25, 26] | `42.5µs`[cite: 25] |
| `test_vfs_disk_io_and_magic_header` | Disk save, load, header & exact quat read[cite: 25, 26] | `545.4µs`[cite: 25] |
| `test_vfs_concurrent_thread_safety` | Multi-threaded concurrent writes across 8 threads[cite: 25, 26] | `661.1µs`[cite: 25] |
| `test_async_vfs_throughput` | Tokio Async VFS: save & load 5,000 quats[cite: 25] | `724.2µs`[cite: 25] |
| `test_vfs_corrupted_disk_header` | Error handling validation for corrupted binary headers[cite: 25, 26] | `730.5µs`[cite: 25] |
| `test_wal_replay_and_commit_truncate` | WAL replay & commit[cite: 25, 26] | `937.9µs`[cite: 25] |
| `test_buffer_batch_swar_and_hamming` | Batch & SWAR operations on 10,000 quats[cite: 25, 26] | `1.3128ms`[cite: 25] |
| `test_mmap_vfs_speed_and_safety` | `MmapQuatFile`: 10,000 random read/writes[cite: 25, 26] | `1.5717ms`[cite: 25] |
| `test_container_performance_and_validity` | `QuatContainer`: 100 files packed & written[cite: 25, 26] | `5.0657ms`[cite: 25] |
| `test_wal_transaction_logging_benchmark` | WAL: 1,000 transaction logs written & committed[cite: 26] | `16.3779ms`[cite: 26] |
| `test_vfs_stress_large_data` | 100,000 quats written, saved, loaded & verified[cite: 26] | `19.6196ms`[cite: 26] |
| `test_assembly_ffi_packing_performance` | ASM FFI pack: 100,000 iterations executed[cite: 26] | `21.2389ms`[cite: 26] |

## Quickstart

Add `qvfs` to your `Cargo.toml`:

```toml
[dependencies]
qvfs = "2.4.0"
```

### Basic Example

```rust
use qvfs::buffer::QuatBuffer;
use qvfs::error::Result;
use qvfs::{CompressedQuatFile, MemoryQuatVFS, Quat, QuatFile, QuatFileSystem};

fn main() -> Result<()> {
    // 1. Initialize VFS
    let mut vfs = MemoryQuatVFS::new();

    // 2. Create a new virtual file
    let file_handle = vfs.create("/sample.quat")?;

    // 3. Write quats (DNA nucleotides: A, C, G, T)
    {
        let mut file = file_handle.lock().unwrap();
        file.write_quat(Quat::Q0)?; // A
        file.write_quat(Quat::Q1)?; // C
        file.write_quat(Quat::Q2)?; // G
        file.write_quat(Quat::Q3)?; // T
    }

    // 4. Read and display written quats
    {
        let mut file = file_handle.lock().unwrap();
        file.seek_quat(0)?;

        while let Some(quat) = file.read_quat()? {
            println!("Read nucleotide: {}", quat.as_char());
        }
    }

    // 5. Save VFS file to real disk (with QVFS header) and reload
    vfs.save_to_disk("/sample.quat", "sample.bin")?;
    let loaded_handle = vfs.load_from_disk("sample.bin", "/loaded_sample.quat")?;

    {
        let loaded_file = loaded_handle.lock().unwrap();
        println!("Loaded file length (quats): {}", loaded_file.len());
    }

    // 6. [v2.0.0] Compressed read-only file usage
    let mut buffer = QuatBuffer::new();
    buffer.push(Quat::Q0);
    buffer.push(Quat::Q3);

    let mut compressed_file = CompressedQuatFile::new(buffer);
    if let Some(q) = compressed_file.read_quat()? {
        println!("Read from compressed file: {}", q.as_char());
    }

    // Cleanup temporary file from disk
    let _ = std::fs::remove_file("sample.bin");

    Ok(())
}
```

### Enterprise & Advanced Modules Example

```rust
use qvfs::mmap_vfs::MmapQuatFile;
use qvfs::async_vfs::AsyncMemoryQuatVFS;
use qvfs::wal::WriteAheadLog;
use qvfs::quat::Quat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Memory-Mapped Quat File (Ultra-fast direct RAM/disk mapping)
    let mut mmap_file = MmapQuatFile::new("mmap_sample.quat")?;
    mmap_file.set_quat(0, Quat::Q3);
    let val = mmap_file.get_quat(0);
    println!("Mmap quat value: {:?}", val);

    // 2. Asynchronous VFS operations using Tokio
    let async_vfs = AsyncMemoryQuatVFS::new("async_storage.bin");
    // async_vfs.save_to_disk_async("/vfs_path", &buffer).await?;

    // 3. Write-Ahead Logging (WAL) for transactions
    let mut wal = WriteAheadLog::new("qvfs.wal")?;
    wal.log_write(0, Quat::Q2)?;
    wal.commit()?;

    Ok(())
}
```

### Advanced Features Example

```rust
fn main() -> Result<(), QuatError> {
    let mut vfs = MemoryQuatVFS::new();
    let _handle = vfs.create("/sample.quat")?;

    // --- 1. Using standard Rust iterators ---
    let mut buffer = QuatBuffer::new();
    buffer.push(Quat::Q0);
    buffer.push(Quat::Q1);
    buffer.push(Quat::Q2);

    for quat in buffer.iter() {
        println!("Quat nucleotide: {}", quat.as_char());
    }

    // --- 2. Quaternary logical operations ---
    let base = Quat::Q1; // 'C'
    let inverted = base.q_not(); // Returns Quat::Q2 ('G')
    let _combined = base.q_and(Quat::Q3);

    println!("Inverted: {}", inverted.as_char());

    // --- 3. VFS logical transformation on-the-fly ---
    vfs.transform_file_logical("/sample.quat", |q| q.q_not())?;

    Ok(())
}
```

### Batch, SWAR & Versioned Persistence Example

```rust
use qvfs::buffer::QuatBuffer;
use qvfs::compressed_file::CompressedQuatFile;
use qvfs::dna::DnaStorage;
use qvfs::error::Result;
use qvfs::quat::Quat;
use qvfs::vfs::{decode_versioned_frame, encode_versioned_frame, QVFS_FLAG_NONE};
use qvfs::wal::WriteAheadLog;

fn main() -> Result<()> {
    // 1. Fast batch DNA encoding & decoding
    let buf = DnaStorage::bytes_to_buffer(b"ACGTACGT")?;
    let dna_str = DnaStorage::buffer_to_dna_fast(&buf);
    assert_eq!(dna_str, "ACGTACGT");

    // 2. SWAR state counting & modulo-4 arithmetic
    let counts = buf.count_states();
    let shifted = buf.batch_add_mod4(&buf).unwrap();
    assert_eq!(counts, [2, 2, 2, 2]);
    assert_eq!(shifted.len_quats, 8);

    // 3. Versioned QVFH v1 frame & RLE compression
    let frame = encode_versioned_frame(&buf.data, buf.len_quats as u64, QVFS_FLAG_NONE)?;
    let (_flags, total_quats, _payload) = decode_versioned_frame(&frame)?;
    assert_eq!(total_quats, 8);

    let rle = CompressedQuatFile::compress_rle_quats(&buf);
    let restored = CompressedQuatFile::decompress_rle_quats(&rle, buf.len_quats)?;
    assert_eq!(restored, buf);

    // 4. WAL transaction & replay
    let mut wal = WriteAheadLog::new("example.wal")?;
    wal.log_begin()?;
    wal.log_write(0, Quat::Q2)?;
    wal.commit()?;

    let mut replay_buf = QuatBuffer::new();
    let _ = wal.replay_committed(&mut replay_buf)?;
    let _ = std::fs::remove_file("example.wal");

    Ok(())
}
```


## Specification

The library encodes states using 2 bits per quat:

| Quat State | Binary | Value | Biological Analog |
| :--- | :--- | :--- | :--- |
| `Q0` | `0b00` | 0 | Adenine (A) |
| `Q1` | `0b01` | 1 | Cytosine (C) |
| `Q2` | `0b10` | 2 | Guanine (G) |
| `Q3` | `0b11` | 3 | Thymine (T) |

# Integration Guide (C, C++, C#)

## 1. C Integration

### Header (`qvfs.h`)
```c
#ifndef QVFS_H
#define QVFS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MemoryQuatVFS MemoryQuatVFS;
typedef struct FileHandle FileHandle;

MemoryQuatVFS* qvfs_vfs_new(void);
void qvfs_vfs_free(MemoryQuatVFS* vfs);
int32_t qvfs_vfs_create(MemoryQuatVFS* vfs, const char* path, FileHandle** out_handle);
void qvfs_file_free(FileHandle* handle);
int32_t qvfs_file_write_quat(FileHandle* handle, uint8_t quat_val);

// Safer & batch quaternary FFI functions (values 0..=3)
int32_t qvfs_file_write_quat_strict(FileHandle* handle, uint8_t quat_val);
int32_t qvfs_file_write_quats_batch(
    FileHandle* handle,
    const uint8_t* quats_ptr,
    size_t quats_len,
    size_t* out_written
);
int32_t qvfs_vfs_exists(const MemoryQuatVFS* vfs, const char* path, int32_t* out_exists);

#ifdef __cplusplus
}
#endif

#endif
```

### Usage (`main.c`)
```c
#include <stdio.h>
#include "qvfs.h"

int main(void) {
    MemoryQuatVFS* vfs = qvfs_vfs_new();
    FileHandle* file = NULL;

    if (qvfs_vfs_create(vfs, "c_test.bin", &file) == 0) {
        qvfs_file_write_quat(file, 1);

        int32_t exists = 0;
        if (qvfs_vfs_exists(vfs, "c_test.bin", &exists) == 0 && exists == 1) {
            uint8_t batch[3] = {0, 2, 3};
            size_t written = 0;
            qvfs_file_write_quats_batch(file, batch, 3, &written);
            qvfs_file_write_quat_strict(file, 1);
        }

        qvfs_file_free(file);
    }

    qvfs_vfs_free(vfs);
    return 0;
}
```

---

## 2. C++ Integration

### Usage (`main.cpp`)
```cpp
#include <iostream>
#include "qvfs.h" // Includes the same C header wrapped in extern "C"

int main() {
    MemoryQuatVFS* vfs = qvfs_vfs_new();
    FileHandle* file = nullptr;

    if (qvfs_vfs_create(vfs, "cpp_test.bin", &file) == 0) {
        qvfs_file_write_quat(file, 2);
        qvfs_file_free(file);
    }

    qvfs_vfs_free(vfs);
    return 0;
}
```

---

## 3. C# Integration (P/Invoke)

### Usage (`Program.cs`)
```csharp
using System;
using System.Runtime.InteropServices;

class Program
{
    private const string LibName = "qvfs";

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr qvfs_vfs_new();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void qvfs_vfs_free(IntPtr vfs);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    private static extern int qvfs_vfs_create(IntPtr vfs, string path, out IntPtr outHandle);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void qvfs_file_free(IntPtr handle);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    private static extern int qvfs_file_write_quat(IntPtr handle, byte quatVal);

    static void Main()
    {
        IntPtr vfs = qvfs_vfs_new();

        if (qvfs_vfs_create(vfs, "cs_test.bin", out IntPtr handle) == 0)
        {
            qvfs_file_write_quat(handle, 3);
            qvfs_file_free(handle);
        }

        qvfs_vfs_free(vfs);
    }
}
```