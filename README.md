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

## Benchmark tests

| Test / Component | Detailed description of what the test does and measures | Benchmark / Time |
| :--- | :--- | :--- |
| **`test_quaternary_logic_operations`** | Calculation and testing of quaternary bitwise/logical operations (`q_not`, `q_and`, `q_or`, `q_xor`)[cite: 21] | **500ns**[cite: 21] |
| **`test_quat_conversions`** | Precise mapping of characters to quaternary values (`A, C, G, T` to `Q0, Q1, Q2, Q3`)[cite: 21] | **1.8µs**[cite: 21] |
| **`test_quat_buffer_iterators`** | Traversal, iteration, and element extraction through packed `QuatBuffer`[cite: 21] | **7.8µs**[cite: 21] |
| **`test_vfs_seek_past_eof`** | Moving pointer past the end of file (EOF), automatic hole padding with Q0, and exact read[cite: 21] | **10.4µs**[cite: 21] |
| **`test_vfs_transform_file_logical`** | Applying in-place logical transformations (e.g., `q_not`) to an entire file on-the-fly within VFS[cite: 21] | **15.2µs**[cite: 21] |
| **`test_compressed_quat_file_read_only`** | Validation of read-only mode for compressed buffer and unauthorized write protection check[cite: 21] | **27.7µs**[cite: 21] |
| **`test_vfs_overwrite_and_delete`** | Complete VFS lifecycle of creation, writing, seeking, overwriting, reading, and deletion[cite: 21] | **19.2µs**[cite: 21] |
| **`test_vfs_disk_io_and_magic_header`** | Disk saving, magic header validation ("QVFS"), and exact loading/reading of quats[cite: 21] | **624.2µs**[cite: 21] |
| **`test_vfs_corrupted_disk_header`** | Error handling and protection validation for corrupted binary headers (`InvalidHeader`)[cite: 21] | **760.5µs**[cite: 21] |
| **`test_async_vfs_throughput`** | Tokio Async VFS: Asynchronous saving and loading of 5,000 quaternary units[cite: 21] | **1.069ms**[cite: 21] |
| **`test_vfs_concurrent_thread_safety`** | Multi-threaded safety with concurrent writes distributed across 8 parallel threads[cite: 21] | **1.2357ms**[cite: 21] |
| **`test_mmap_vfs_speed_and_safety`** | Memory-mapped file (`MmapQuatFile`): Execution of 10,000 random reads and writes[cite: 21] | **1.6681ms**[cite: 21] |
| **`test_container_performance_and_validity`** | Single-file container (`QuatContainer`): Packaging and writing 100 virtual files into a single container[cite: 21] | **4.8158ms**[cite: 21] |
| **`test_wal_transaction_logging`** | Transactional WAL logs: Writing and committing 1,000 transaction logs[cite: 21] | **14.39ms**[cite: 21] |
| **`test_vfs_stress_large_data`** | Stress test with 100,000 quats: Writing, saving to disk, loading, and byte-by-byte verification[cite: 21] | **18.8233ms**[cite: 21] |
| **`test_assembly_ffi_packing_performance`** | ASM FFI layer: Execution of 100,000 raw byte packaging iterations[cite: 21] | **19.4141ms**[cite: 21] |
| **`test_ffi_batch_read`** | FFI testing of batch reading an exact number of quats from a memory buffer | - |
| **`test_ffi_null_safety`** | FFI testing of safety against NULL pointers to prevent application crashes | - |
| **`test_ffi_vfs_lifecycle`** | Complete lifecycle test of the FFI layer (creation, allocation, operation, and safe deallocation) | - |

## Quickstart

Add `qvfs` to your `Cargo.toml`:

```toml
[dependencies]
qvfs = "2.3.0"
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