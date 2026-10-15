use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

// ============================================================================
// 1. Foreign C Declarations (Calling C from Rust)
// ============================================================================

extern "C" {
    fn c_compute_checksum(data: *const u8, len: usize, out_checksum: *mut u32) -> i32;
    fn c_apply_mask(data: *mut u8, len: usize, mask: u8) -> i32;
}

/// Computes a djb2 rolling checksum via the linked C library.
pub fn compute_checksum(data: &[u8]) -> Result<u32, &'static str> {
    let mut out: u32 = 0;
    // SAFETY: We pass a valid pointer and slice length. The C function does not
    // retain the pointer or mutate `data`.
    let ret = unsafe { c_compute_checksum(data.as_ptr(), data.len(), &mut out) };
    if ret == 0 {
        Ok(out)
    } else {
        Err("C checksum computation failed")
    }
}

/// Applies an in-place XOR mask to a byte buffer via the linked C library.
pub fn apply_mask(buffer: &mut [u8], mask: u8) -> Result<(), &'static str> {
    // SAFETY: We pass a valid mutable pointer and length. The C function mutates
    // the slice in-place without exceeding the bounds.
    let ret = unsafe { c_apply_mask(buffer.as_mut_ptr(), buffer.len(), mask) };
    if ret == 0 {
        Ok(())
    } else {
        Err("C mask application failed")
    }
}

// ============================================================================
// 2. Exported C-ABI Functions (Calling Rust from C)
// ============================================================================

/// Transforms a C string by reversing and capitalizing it.
/// Returns a freshly allocated C string which the caller MUST free with `rust_free_string`.
/// If `input` is null or invalid UTF-8, returns NULL.
///
/// # Safety
/// `input` must point to a valid null-terminated C string if non-null.
#[no_mangle]
pub unsafe extern "C" fn rust_transform_string(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: We checked that `input` is non-null. The caller contracts that it is null-terminated.
    let c_str = unsafe { CStr::from_ptr(input) };
    let text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // Transform: reverse characters and uppercase them
    let transformed: String = text.chars().rev().flat_map(|c| c.to_uppercase()).collect();

    // Convert to CString and transfer ownership to caller
    match CString::new(transformed) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Calculates the length of a null-terminated C string using Rust.
/// Returns -1 if `s` is NULL.
///
/// # Safety
/// If non-null, `s` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rust_string_length(s: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(s) };
    c_str.to_bytes().len() as c_int
}

/// Frees a string previously returned by `rust_transform_string`.
///
/// # Safety
/// `ptr` must have been obtained from `CString::into_raw` (e.g. from `rust_transform_string`),
/// and must not be used after this call. Passing NULL is safe and a no-op.
#[no_mangle]
pub unsafe extern "C" fn rust_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: Reclaims ownership of the pointer so it drops through Rust's allocator.
        drop(unsafe { CString::from_raw(ptr) });
    }
}
