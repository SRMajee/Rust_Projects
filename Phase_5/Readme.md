# Phase 5: Low-Level OS & Interoperability

This phase explores raw pointers, uninitialized memory handling, unsafe Rust, C-ABI bindings, and cross-language interoperability.

## Projects in this Phase

1. **Custom Stack-Allocated Vector (`ArrayVec`)**
   - **Focus**: Fixed-capacity vectors, `MaybeUninit<T>`, inline array storage on the stack without heap allocation, and manual drop tracking.
   - **Status**: Completed (Zero heap allocation, const-generic capacity, full slice `Deref`/`DerefMut` ergonomics, manual `Drop` & `IntoIter` tracking, 100% verified with Miri).

2. **Dynamic Linking & FFI (C-ABI Wrapper)**
   - **Focus**: `extern "C"`, `#[no_mangle]`, bidirectional foreign function interfaces, bridging Rust with C libraries via `build.rs` and the `cc` crate, wrapping raw pointers with safe abstractions, and cross-language string lifecycle management (`CString::into_raw` & `from_raw`).
   - **Status**: Completed (Dual `cdylib` and `rlib` targets, compiles to native dynamic library `.dll` / `.so`, fully tested FFI boundaries, zero memory leaks).
