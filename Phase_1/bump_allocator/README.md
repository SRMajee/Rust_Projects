# Custom Bump Allocator

A high-performance, thread-safe memory allocator that carves out chunks sequentially from a pre-allocated contiguous buffer.

## Key Features

- **Zero internal heap allocations** during the allocation hot-path.
- **Strict memory alignment**: Uses bitwise power-of-two alignment calculation:

  ```rust
  aligned_addr = (current_addr + align - 1) & !(align - 1)
  ```

- **Thread-safe (`Sync` & `Send`)**: Implemented with lock-free `AtomicUsize` and compare-and-swap (`compare_exchange_weak`) loops.
- **Dual usage modes**:
  1. **Scoped Typed Arena**: `alloc_val<T>(val: T) -> Option<&mut T>` returns lifetime-bounded references without manual pointer management.
  2. **`std::alloc::GlobalAlloc`**: Can be used directly as Rust's `#[global_allocator]` across your application.
- **Bulk deallocation**: O(1) deallocation via `reset()` or automatically on `Drop`.

## Testing

Run the test suite:

```bash
cargo test
```

Includes:

- Primitive type allocation
- Alignment verification (testing 1-byte, 8-byte, and 64-byte `repr(align(64))` structs)
- Out-of-memory bounds checking
- Zero-sized types (ZST) handling
- Multithreaded concurrent allocation stress test
- Integration test using `#[global_allocator]` with standard library `Box`, `Vec`, and `String`
