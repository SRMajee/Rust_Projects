# Phase 5: Low-Level OS & Interoperability

This phase explores raw pointers, uninitialized memory handling, unsafe Rust, C-ABI bindings, and cross-language interoperability.

## Projects in this Phase

1. **Custom Stack-Allocated Vector (`ArrayVec`)**
   - **Focus**: Fixed-capacity vectors, `MaybeUninit<T>`, inline array storage on the stack without heap allocation, and manual drop tracking.

2. **Dynamic Linking & FFI (C-ABI Wrapper)**
   - **Focus**: `extern "C"`, `#[no_mangle]`, foreign function interfaces, bridging Rust with C libraries, safely wrapping raw pointers, and ABI compatibility.
