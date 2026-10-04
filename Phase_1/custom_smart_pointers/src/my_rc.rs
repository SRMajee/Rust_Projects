use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::cell::Cell;
use std::fmt;
use std::ops::Deref;
use std::ptr::NonNull;

/// Internal heap control block holding the reference count and the managed value.
struct RcBox<T> {
    strong_count: Cell<usize>,
    value: T,
}

/// A custom reference-counting smart pointer for single-threaded shared ownership.
///
/// Analogous to `std::rc::Rc<T>`. Multiple `MyRc` pointers can reference the same allocation.
/// When the last `MyRc` is dropped, the inner `value` is dropped in place and the heap memory is freed.
///
/// # Thread Safety
/// `MyRc` is explicitly **NOT** `Send` or `Sync` because it uses non-atomic reference counting (`Cell<usize>`).
pub struct MyRc<T> {
    ptr: NonNull<RcBox<T>>,
}

impl<T> MyRc<T> {
    /// Constructs a new `MyRc<T>` containing `value`.
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<RcBox<T>>();

        // Even if T is a ZST, RcBox has strong_count (usize), so layout.size() > 0 always.
        let raw = unsafe { alloc(layout) } as *mut RcBox<T>;
        let non_null = match NonNull::new(raw) {
            Some(p) => p,
            None => handle_alloc_error(layout),
        };

        unsafe {
            std::ptr::write(
                non_null.as_ptr(),
                RcBox {
                    strong_count: Cell::new(1),
                    value,
                },
            );
        }

        Self { ptr: non_null }
    }

    /// Returns the current number of active `MyRc` pointers to this allocation.
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.ptr.as_ref().strong_count.get() }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T> Clone for MyRc<T> {
    #[inline]
    fn clone(&self) -> Self {
        let count = unsafe { &self.ptr.as_ref().strong_count };
        let current = count.get();
        // Guard against overflow
        assert!(current != usize::MAX, "Reference count overflow");
        count.set(current + 1);

        Self { ptr: self.ptr }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        let count = unsafe { &self.ptr.as_ref().strong_count };
        let current = count.get();

        if current > 1 {
            count.set(current - 1);
        } else {
            // Count is 1; this is the last reference.
            let layout = Layout::new::<RcBox<T>>();
            unsafe {
                // 1. Drop the value in place
                std::ptr::drop_in_place(&mut (*self.ptr.as_ptr()).value);

                // 2. Deallocate the control block
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

// Notice: We intentionally DO NOT implement Send or Sync for MyRc<T>
// because strong_count uses unsynchronized Cell<usize>.

impl<T: fmt::Debug> fmt::Debug for MyRc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MyRc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
