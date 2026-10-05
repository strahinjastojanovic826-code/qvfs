# qvfs

`qvfs` is a Rust-based Quaternary Virtual File System (QVFS) simulator. It models file system operations using 4-state (quaternary) logic representation (Q0, Q1, Q2, Q3) mapped to 2-bit quantum/biological states (e.g., Adenine, Cytosine, Guanine, Thymine).

## Overview

Unlike standard binary file systems operating strictly on bits (0 and 1), `qvfs` implements storage and state manipulation on 2-bit quats. This crate provides data structures and a virtual execution context for experimenting with multi-state logic systems.

## Features

* **Quat Primitive Types:** Core 4-state primitives (Q0, Q1, Q2, Q3) with bitwise conversion utilities.
* **Biomorphic Mapping:** Built-in representation of DNA bases (A, C, G, T) mapped directly to 2-bit boundaries.
* **In-Memory VFS:** Thread-safe virtual file system (`MemoryQuatVFS`) supporting file creation, modification, deletion, and directory listings (`list_dir`).
* **Binary Persistence with Magic Header:** Custom file format with a mandatory 4-byte `QVFS` header and high-performance serialization (`save_to_disk` / `load_from_disk`).
* **Compressed File Extensions:** Support for read-only compressed quat structures via `CompressedQuatFile` and buffer-based search operations (`QuatBuffer`).
* **Custom Error Handling:** Strongly-typed `QuatError` enum for exact failure diagnosis (header corruption, invalid characters, permission issues).

## Quickstart

Add `qvfs` to your `Cargo.toml`:

```toml
[dependencies]
qvfs = "2.1.0"
```
## Licensing

QVFS is dual-licensed:

1. **Open Source (AGPL-3.0)**: Free to use for open-source projects under the terms of the GNU Affero General Public License v3.0.
2. **Commercial License**: If you wish to use QVFS in proprietary software or closed-source commercial products without AGPL-3.0 restrictions, you must acquire a commercial license.

For commercial licensing inquiries, contact: `strahinjastojanovic826@gmail.com`

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
