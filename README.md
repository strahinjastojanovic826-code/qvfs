# qvfs

`qvfs` is a Rust-based Quaternary Virtual File System (QVFS) simulator. It models file system operations using 4-state (quaternary) logic representation (Q0, Q1, Q2, Q3) mapped to 2-bit quantum/biological states (e.g., Adenine, Cytosine, Guanine, Thymine).

## Overview

Unlike standard binary file systems operating strictly on bits (0 and 1), `qvfs` implements storage and state manipulation on 2-bit quats. This crate provides data structures and a virtual execution context for experimenting with multi-state logic systems.

## Features

* **Quat Primitive Types:** Core 4-state primitives with bitwise conversion utilities.
* **Biomorphic Mapping:** Built-in representation of DNA bases (A, C, G, T) mapped to 2-bit boundaries.
* **In-Memory VFS:** Thread-safe virtual file system primitives for 4-state data structures.

## Quickstart

Add `qvfs` to your `Cargo.toml`:

```toml
[dependencies]
qvfs = "0.1.0"
```

### Basic Example

```rust
use qvfs::Quat;

fn main() {
    // Convert byte value mask into a Quat primitive
    let quat = Quat::from_u8(0b01);
    
    assert_eq!(quat, Quat::Q1);
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