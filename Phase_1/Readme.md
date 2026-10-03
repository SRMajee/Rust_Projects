# Phase 1: Foundations & Memory Management

This phase focuses on Rust's core memory model, ownership, borrowing, lifetimes, raw pointers, and custom allocators.

## Projects in this Phase

1. **[Zero-Copy String Tokenizer / Scanner](./zero_copy_tokenizer)** ✅
   - **Focus**: Slices, lifetimes (`'a`), zero-copy parsing, and iterator patterns without heap reallocations.

2. **Custom Smart Pointer (`MyBox` & `MyRc`)**
   - **Focus**: Custom dereferencing (`Deref`, `DerefMut`), resource cleanup (`Drop`), reference counting, and interior mutability basics.

3. **[NEW] Custom Bump Allocator**
   - **Focus**: Low-level memory layout, pointer arithmetic, memory alignment, `std::alloc::Layout`, and arena-style allocation.
