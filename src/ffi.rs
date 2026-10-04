use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

use crate::quat::Quat;
use crate::traits::{QuatFile, QuatFileSystem};
use crate::vfs::{FileHandle, MemoryQuatVFS};

// -----------------------------------------------------------------------------
// VFS (MemoryQuatVFS)
// -----------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn qvfs_vfs_new() -> *mut MemoryQuatVFS {
    Box::into_raw(Box::new(MemoryQuatVFS::new()))
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_free(vfs: *mut MemoryQuatVFS) {
    if !vfs.is_null() {
        drop(Box::from_raw(vfs));
    }
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_create(
    vfs: *mut MemoryQuatVFS,
    path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_open(
    vfs: *mut MemoryQuatVFS,
    path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_delete(vfs: *mut MemoryQuatVFS, path: *const c_char) -> c_int {
    if vfs.is_null() || path.is_null() {
        return -1;
    }
    let c_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    if (*vfs).delete(c_str).is_ok() { 0 } else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_save_to_disk(
    vfs: *mut MemoryQuatVFS,
    vfs_path: *const c_char,
    real_disk_path: *const c_char,
) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_vfs_load_from_disk(
    vfs: *mut MemoryQuatVFS,
    real_disk_path: *const c_char,
    vfs_path: *const c_char,
    out_handle: *mut *mut FileHandle,
) -> c_int {
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
}

// -----------------------------------------------------------------------------
// FileHandle (QuatFile)
// -----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_free(handle: *mut FileHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_read_quat(
    handle: *mut FileHandle,
    out_quat: *mut u8,
    out_has_val: *mut c_int,
) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_write_quat(handle: *mut FileHandle, quat_val: u8) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_seek(
    handle: *mut FileHandle,
    offset: usize,
    out_new_offset: *mut usize,
) -> c_int {
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
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_len(handle: *mut FileHandle, out_len: *mut usize) -> c_int {
    if handle.is_null() || out_len.is_null() {
        return -1;
    }
    let file = match (*handle).lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    *out_len = file.len();
    0
}

#[no_mangle]
pub unsafe extern "C" fn qvfs_file_read_exact_quats(
    handle: *mut FileHandle,
    out_buf: *mut u8,
    buf_len: usize,
    out_read_count: *mut usize,
) -> c_int {
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
}