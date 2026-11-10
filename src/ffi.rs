use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

use crate::quat::Quat;
use crate::traits::{QuatFile, QuatFileSystem};
use crate::vfs::{FileHandle, MemoryQuatVFS};

// -----------------------------------------------------------------------------
// VFS (MemoryQuatVFS)
// -----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn qvfs_vfs_new() -> *mut MemoryQuatVFS {
    Box::into_raw(Box::new(MemoryQuatVFS::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_free(vfs: *mut MemoryQuatVFS) { unsafe {
    if !vfs.is_null() {
        drop(Box::from_raw(vfs));
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_create(
    vfs: *mut MemoryQuatVFS,
    path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int { unsafe {
    if vfs.is_null() || path.is_null() || out_handle.is_null() {
        return -1;
    }
    let c_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match (*vfs).create(c_str) {
        Ok(handle) => {
            *out_handle = Box::into_raw(Box::new(handle));
            0
        }
        Err(_) => -1,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_open(
    vfs: *mut MemoryQuatVFS,
    path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int { unsafe {
    if vfs.is_null() || path.is_null() || out_handle.is_null() {
        return -1;
    }
    let c_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match (*vfs).open(c_str) {
        Ok(handle) => {
            *out_handle = Box::into_raw(Box::new(handle));
            0
        }
        Err(_) => -1,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_delete(vfs: *mut MemoryQuatVFS, path: *const c_char) -> c_int { unsafe {
    if vfs.is_null() || path.is_null() {
        return -1;
    }
    let c_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    if (*vfs).delete(c_str).is_ok() { 0 } else { -1 }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_save_to_disk(
    vfs: *mut MemoryQuatVFS,
    vfs_path: *const c_char,
    real_disk_path: *const c_char,
) -> c_int { unsafe {
    if vfs.is_null() || vfs_path.is_null() || real_disk_path.is_null() {
        return -1;
    }
    let v_path = match CStr::from_ptr(vfs_path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let r_path = match CStr::from_ptr(real_disk_path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    if (*vfs).save_to_disk(v_path, r_path).is_ok() { 0 } else { -1 }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_vfs_load_from_disk(
    vfs: *mut MemoryQuatVFS,
    real_disk_path: *const c_char,
    vfs_path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int { unsafe {
    if vfs.is_null() || real_disk_path.is_null() || vfs_path.is_null() || out_handle.is_null() {
        return -1;
    }
    let r_path = match CStr::from_ptr(real_disk_path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let v_path = match CStr::from_ptr(vfs_path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match (*vfs).load_from_disk(r_path, v_path) {
        Ok(handle) => {
            *out_handle = Box::into_raw(Box::new(handle));
            0
        }
        Err(_) => -1,
    }
}}

// -----------------------------------------------------------------------------
// FileHandle (QuatFile)
// -----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_free(handle: *mut FileHandle) { unsafe {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_read_quat(
    handle: *mut FileHandle,
    out_quat: *mut u8,
    out_has_val: *mut c_int,
) -> c_int { unsafe {
    if handle.is_null() || out_quat.is_null() || out_has_val.is_null() {
        return -1;
    }
    let mut file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    match file.read_quat() {
        Ok(Some(q)) => {
            *out_quat = q as u8;
            *out_has_val = 1;
            0
        }
        Ok(None) => {
            *out_has_val = 0;
            0
        }
        Err(_) => -1,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_write_quat(handle: *mut FileHandle, quat_val: u8) -> c_int { unsafe {
    if handle.is_null() || quat_val > 3 {
        return -1;
    }
    let quat = match quat_val {
        0 => Quat::Q0,
        1 => Quat::Q1,
        2 => Quat::Q2,
        _ => Quat::Q3,
    };
    let mut file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    if file.write_quat(quat).is_ok() { 0 } else { -1 }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_seek(
    handle: *mut FileHandle,
    offset: usize,
    out_new_offset: *mut usize,
) -> c_int { unsafe {
    if handle.is_null() || out_new_offset.is_null() {
        return -1;
    }
    let mut file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    match file.seek_quat(offset) {
        Ok(pos) => {
            *out_new_offset = pos;
            0
        }
        Err(_) => -1,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_len(handle: *mut FileHandle, out_len: *mut usize) -> c_int { unsafe {
    if handle.is_null() || out_len.is_null() {
        return -1;
    }
    let file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    *out_len = file.len();
    0
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_file_read_exact_quats(
    handle: *mut FileHandle,
    out_buf: *mut u8,
    buf_len: usize,
    out_read_count: *mut usize,
) -> c_int { unsafe {
    if handle.is_null() || out_buf.is_null() || out_read_count.is_null() {
        return -1;
    }
    let mut file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };

    let mut temp_quats = vec![Quat::Q0; buf_len];
    match file.read_exact_quats(&mut temp_quats) {
        Ok(count) => {
            let dest = slice::from_raw_parts_mut(out_buf, count);
            for i in 0..count {
                dest[i] = temp_quats[i] as u8;
            }
            *out_read_count = count;
            0
        }
        Err(_) => -1,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_asm_pack_buffer(
    src: *const u8,
    src_len: usize,
    dest: *mut u8,
    dest_len: usize,
) -> i32 { unsafe {
    if src.is_null() || dest.is_null() {
        return -1; // Null pointer error code for ASM
    }

    let required_dest_len = (src_len + 3) / 4;
    if dest_len < required_dest_len {
        return -2; // Buffer too small error code
    }

    let src_slice = std::slice::from_raw_parts(src, src_len);
    let dest_slice = std::slice::from_raw_parts_mut(dest, required_dest_len);

    // Clear destination bytes first
    for b in dest_slice.iter_mut() {
        *b = 0;
    }

    for (i, &val) in src_slice.iter().enumerate() {
        let byte_index = i / 4;
        let shift = 6 - ((i % 4) * 2);
        dest_slice[byte_index] |= (val & 0b11) << shift;
    }

    0 // Success status code for Assembly registers
}}

/// Raw, flat function for Assembly to unpack 2-bit quats into a flat destination array
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qvfs_asm_unpack_buffer(
    src: *const u8,
    src_len: usize,
    quat_count: usize,
    dest: *mut u8,
    dest_len: usize,
) -> i32 { unsafe {
    if src.is_null() || dest.is_null() {
        return -1;
    }
    if dest_len < quat_count {
        return -2;
    }

    let src_slice = std::slice::from_raw_parts(src, src_len);
    let dest_slice = std::slice::from_raw_parts_mut(dest, quat_count);

    for i in 0..quat_count {
        let byte_index = i / 4;
        if byte_index >= src_slice.len() {
            break;
        }
        let shift = 6 - ((i % 4) * 2);
        let val = (src_slice[byte_index] >> shift) & 0b11;
        dest_slice[i] = val;
    }

    0
}}