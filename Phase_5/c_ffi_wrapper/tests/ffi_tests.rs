use c_ffi_wrapper::{
    apply_mask, compute_checksum, rust_free_string, rust_string_length, rust_transform_string,
};
use std::ffi::{CStr, CString};
use std::ptr;

#[test]
fn test_c_checksum_wrapper() {
    let data = b"hello, ffi";
    let checksum = compute_checksum(data).expect("checksum failed");
    assert_ne!(checksum, 0);

    // Identical data must produce identical checksum
    let checksum2 = compute_checksum(data).expect("checksum failed");
    assert_eq!(checksum, checksum2);

    // Empty buffer works as well
    let empty_sum = compute_checksum(b"").expect("empty checksum failed");
    assert_eq!(empty_sum, 5381); // djb2 initial value
}

#[test]
fn test_c_apply_mask_wrapper() {
    let mut data = *b"Hello, C-ABI!";
    let original = data;

    // Mask with 0xAA
    apply_mask(&mut data, 0xAA).expect("mask failed");
    assert_ne!(data, original);

    // Re-applying the same XOR mask recovers original data
    apply_mask(&mut data, 0xAA).expect("unmask failed");
    assert_eq!(data, original);
}

#[test]
fn test_rust_exported_string_length() {
    let c_str = CString::new("Rust FFI Boundary").unwrap();
    let len = unsafe { rust_string_length(c_str.as_ptr()) };
    assert_eq!(len, 17);

    // NULL pointer handling
    let null_len = unsafe { rust_string_length(ptr::null()) };
    assert_eq!(null_len, -1);
}

#[test]
fn test_rust_exported_transform_string() {
    let input = CString::new("rustacean").unwrap();

    let out_ptr = unsafe { rust_transform_string(input.as_ptr()) };
    assert!(!out_ptr.is_null());

    let result_str = unsafe { CStr::from_ptr(out_ptr).to_str().unwrap() };
    assert_eq!(result_str, "NAECATSUR"); // Reversed and uppercased

    // Cleanup via our exported deallocator
    unsafe { rust_free_string(out_ptr) };
}

#[test]
fn test_rust_exported_transform_null_or_free_null() {
    // Transforming null should safely return null
    let out = unsafe { rust_transform_string(ptr::null()) };
    assert!(out.is_null());

    // Freeing null must be a safe no-op
    unsafe { rust_free_string(ptr::null_mut()) };
}
