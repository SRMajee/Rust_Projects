use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// A custom, uniquely-owned smart pointer wrapping heap-allocated memory.
///
/// Analogous to `std::boxed::Box<T>`. Memory is allocated using [`std::alloc::alloc`]
/// and automatically deallocated via [`std::alloc::dealloc`] on [`Drop`].
///
/// # Invariants
/// - `ptr` always points to a valid, uniquely owned, initialized `T` on the heap,
///   unless `T` is a Zero-Sized Type (ZST), in which case `NonNull::dangling()` is used.
pub struct MyBox<T> {
    ptr: NonNull<T>,
}

impl<T> MyBox<T> {
    /// Allocates memory on the heap and places `value` into it.
    ///
    /// Handles zero-sized types (ZSTs) correctly without allocating heap memory.
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();

        let ptr = if layout.size() == 0 {
            // For ZSTs, do not call alloc (doing so with size 0 is undefined behavior / unsupported)
            NonNull::dangling()
        } else {
            // Allocate heap memory for T
            let raw = unsafe { alloc(layout) } as *mut T;
            let non_null = match NonNull::new(raw) {
                Some(p) => p,
                None => handle_alloc_error(layout),
            };

            // Safely write the value into uninitialized heap memory
            unsafe {
                std::ptr::write(non_null.as_ptr(), value);
            }

            non_null
        };

        Self { ptr }
    }

    /// Consumes the `MyBox<T>`, returning the wrapped value without running `Drop`
    /// on the inner `T`, and deallocates the heap memory.
    pub fn into_inner(b: Self) -> T {
        // Prevent Drop from running on `b`
        let b = std::mem::ManuallyDrop::new(b);
        let layout = Layout::new::<T>();

        unsafe {
            // Read the value out of the pointer
            let val = std::ptr::read(b.ptr.as_ptr());

            // Deallocate if non-ZST
            if layout.size() > 0 {
                dealloc(b.ptr.as_ptr() as *mut u8, layout);
            }

            val
        }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for MyBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        let layout = Layout::new::<T>();

        unsafe {
            // 1. Drop the value in place (runs any custom destructor T might have)
            std::ptr::drop_in_place(self.ptr.as_ptr());

            // 2. Deallocate heap memory if not a ZST
            if layout.size() > 0 {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

// Send and Sync implementations:
// A MyBox<T> can be sent across threads if T is Send, and shared if T is Sync.
unsafe impl<T: Send> Send for MyBox<T> {}
unsafe impl<T: Sync> Sync for MyBox<T> {}

impl<T: fmt::Debug> fmt::Debug for MyBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MyBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
