# Custom Smart Pointers (`MyBox` & `MyRc`)

A foundational, zero-dependency implementation of custom smart pointers in Rust, built directly on top of low-level memory allocators (`std::alloc`), raw pointers (`NonNull<T>`), and Rust's Resource Acquisition Is Initialization (RAII) ownership model.

Part of **Phase 1: Foundations & Memory Management**.

---

## 🎯 Objectives & Concepts Covered

- **Low-level Memory Allocation**: Using `std::alloc::{alloc, dealloc, Layout, handle_alloc_error}`.
- **Raw Pointers & Covariance**: Encapsulating raw memory with `std::ptr::NonNull<T>` to gain non-null optimization (null pointer optimization / niche filling, e.g. `size_of::<Option<MyBox<T>>>() == size_of::<usize>()`).
- **Zero-Sized Types (ZSTs)**: Handling zero-byte structures safely with `NonNull::dangling()` without passing size 0 to the global allocator.
- **RAII & Destructors**: Custom `Drop` implementations executing `std::ptr::drop_in_place` followed by layout-aware `dealloc`.
- **Deref Coercion**: Implementing `Deref` and `DerefMut` for ergonomic pointer transparent access.
- **Single-Threaded Reference Counting**: Control blocks (`RcBox<T>`), interior mutability with `Cell<usize>`, reference counting invariants, and explicit non-thread-safe semantics (`!Send`, `!Sync`).

---

## 📐 Architecture & Memory Layout

### 1. `MyBox<T>` (Unique Heap Ownership)

```text
Stack                      Heap
┌──────────────┐          ┌──────────────┐
│  MyBox<T>    │ ───────> │      T       │
│  ptr: NonNull│          │ (raw memory) │
└──────────────┘          └──────────────┘
```

- **Allocation**: `Layout::new::<T>()` requests the exact size and alignment.
- **ZST Handling**: If `Layout::new::<T>().size() == 0`, heap allocation is bypassed and `NonNull::dangling()` is used.
- **Deallocation**:
  1. `std::ptr::drop_in_place(self.ptr.as_ptr())` invokes the inner type's destructor.
  2. `std::alloc::dealloc(...)` frees the backing bytes back to the system.

### 2. `MyRc<T>` (Shared Heap Ownership)

```text
Stack (rc1)
┌──────────────┐
│  MyRc<T>     │ ───┐
└──────────────┘    │
                    ▼      Heap
Stack (rc2)       ┌────────────────────────┐
┌──────────────┐  │  RcBox<T>              │
│  MyRc<T>     │ ─┼─► strong_count: Cell(2)│
└──────────────┘  │   value: T             │
                  └────────────────────────┘
```

- **Control Block**: `RcBox<T>` bundles `strong_count: Cell<usize>` with the data payload `value: T`.
- **Cloning**: Clones do not copy `value`; they increment `strong_count` via interior mutability in $O(1)$.
- **Drop**: Each drop decrements `strong_count`. When it reaches 0:
  1. `std::ptr::drop_in_place(&mut (*ptr).value)` drops the value.
  2. `std::alloc::dealloc` frees the `RcBox<T>`.

---

## 🚀 Usage Examples

### `MyBox<T>`

```rust
use custom_smart_pointers::MyBox;

// Basic heap allocation
let mut b = MyBox::new(String::from("Hello"));

// DerefMut allows calling methods on String directly
b.push_str(", world!");
assert_eq!(*b, "Hello, world!");

// Extract inner value without dropping
let inner: String = MyBox::into_inner(b);
assert_eq!(inner, "Hello, world!");
```

### `MyRc<T>`

```rust
use custom_smart_pointers::MyRc;
use std::cell::RefCell;

// Shared ownership over state
let shared_data = MyRc::new(RefCell::new(vec![1, 2, 3]));
let worker_clone = shared_data.clone();

assert_eq!(MyRc::strong_count(&shared_data), 2);

// Mutate via interior mutability
worker_clone.borrow_mut().push(4);

assert_eq!(*shared_data.borrow(), vec![1, 2, 3, 4]);
```

---

## 🧪 Testing & Verification

Comprehensive unit tests in [`src/tests.rs`](file:///c:/DRIVE%20D/Rust_Projects/Phase_1/custom_smart_pointers/src/tests.rs) verify:

1. **Destructor Execution Spies (`DropSpy`)**: Verifies that inner types have their destructors triggered **exactly once**, precisely when the owner (or last `MyRc` clone) goes out of scope.
2. **Zero-Sized Types**: Both `MyBox<()>` and `MyRc<()>` are tested to ensure zero memory corruption or invalid allocations.
3. **Interior Mutability**: Using `RefCell` within `MyRc`.
4. **`into_inner`**: Consumes the box, returning ownership of `T` without double drops.

Run the test suite:

```powershell
cargo test
```
