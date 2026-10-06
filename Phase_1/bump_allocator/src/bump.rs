use std::alloc::{alloc, dealloc, GlobalAlloc, Layout};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A thread-safe, fixed-size bump allocator (arena allocator).
///
/// Allocations carve out consecutive chunks of memory from a pre-allocated buffer
/// by bumping an atomic offset forward. Deallocation of individual chunks is a no-op;
/// all memory is reclaimed simultaneously when the allocator is dropped or when `reset()` is called.
enum Storage {
    Dynamic { ptr: NonNull<u8>, capacity: usize },
    Static { ptr: *mut u8, capacity: usize },
}

/// A thread-safe, fixed-size bump allocator (arena allocator).
///
/// Allocations carve out consecutive chunks of memory from a pre-allocated buffer
/// by bumping an atomic offset forward. Deallocation of individual chunks is a no-op;
/// all memory is reclaimed simultaneously when the allocator is dropped or when `reset()` is called.
pub struct BumpAllocator {
    storage: Storage,
    offset: AtomicUsize,
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Creates a new `BumpAllocator` with dynamic heap-allocated capacity.
    ///
    /// # Panics
    /// Panics if `capacity == 0` or if the backing memory allocation fails.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");

        let align = std::mem::align_of::<usize>().max(16);
        let layout = Layout::from_size_align(capacity, align)
            .expect("Invalid layout for arena buffer");

        let raw = unsafe { alloc(layout) };
        let ptr = NonNull::new(raw).expect("Failed to allocate memory for BumpAllocator arena");

        Self {
            storage: Storage::Dynamic { ptr, capacity },
            offset: AtomicUsize::new(0),
        }
    }

    /// Creates a `BumpAllocator` backed by a static raw memory buffer.
    /// Useful for `#[global_allocator]` static definitions without runtime allocation.
    ///
    /// # Safety
    /// The caller must ensure that `ptr` points to valid, aligned memory of at least `capacity` bytes
    /// and that the memory remains valid for the static lifetime without aliasing mutable references.
    pub const unsafe fn from_raw_parts(ptr: *mut u8, capacity: usize) -> Self {
        Self {
            storage: Storage::Static { ptr, capacity },
            offset: AtomicUsize::new(0),
        }
    }

    /// Returns the base pointer of the arena.
    #[inline]
    fn base_ptr(&self) -> *mut u8 {
        match &self.storage {
            Storage::Dynamic { ptr, .. } => ptr.as_ptr(),
            Storage::Static { ptr, .. } => *ptr,
        }
    }

    /// Returns the total capacity of the allocator in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        match &self.storage {
            Storage::Dynamic { capacity, .. } | Storage::Static { capacity, .. } => *capacity,
        }
    }

    /// Returns the current number of allocated/bumped bytes.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Resets the allocator bump pointer back to 0.
    ///
    /// # Safety
    /// This requires `&mut self`, guaranteeing exclusive access.
    /// Callers must ensure that no active references or pointers into the arena
    /// are accessed after calling `reset()`.
    pub fn reset(&mut self) {
        self.offset.store(0, Ordering::Relaxed);
    }

    /// Allocates an instance of `T` in the arena and returns an exclusive reference.
    ///
    /// The reference's lifetime is bound to the allocator (`&'a self`).
    ///
    /// Returns `None` if the allocator has insufficient space or alignment cannot be satisfied.
    pub fn alloc_val<T>(&self, val: T) -> Option<&mut T> {
        let layout = Layout::new::<T>();
        let raw = self.alloc_raw(layout)?;

        unsafe {
            let typed_ptr = raw as *mut T;
            ptr::write(typed_ptr, val);
            Some(&mut *typed_ptr)
        }
    }

    /// Raw allocation helper that complies with memory layout alignment.
    /// Returns `None` on out-of-memory.
    pub fn alloc_raw(&self, layout: Layout) -> Option<*mut u8> {
        if layout.size() == 0 {
            // For zero-sized types, return a properly aligned non-null dangling pointer.
            return Some(ptr::NonNull::<u8>::dangling().as_ptr());
        }

        let align = layout.align();
        let size = layout.size();
        let base_addr = self.base_ptr() as usize;

        let mut current_offset = self.offset.load(Ordering::Relaxed);

        loop {
            let current_addr = base_addr.checked_add(current_offset)?;
            
            // Calculate padding needed to align current_addr to `align`
            // (align is guaranteed to be a power of 2)
            let aligned_addr = (current_addr.checked_add(align - 1)?) & !(align - 1);
            let padding = aligned_addr - current_addr;

            let new_offset = current_offset
                .checked_add(padding)?
                .checked_add(size)?;

            if new_offset > self.capacity() {
                return None; // Out of memory
            }

            // Attempt atomic bump
            match self.offset.compare_exchange_weak(
                current_offset,
                new_offset,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let result_ptr = (base_addr + current_offset + padding) as *mut u8;
                    return Some(result_ptr);
                }
                Err(actual_offset) => {
                    current_offset = actual_offset;
                }
            }
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_raw(layout).unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocators do not support individual deallocation.
        // Memory is reclaimed in bulk on reset() or Drop.
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        if let Storage::Dynamic { ptr, capacity } = &self.storage {
            let align = std::mem::align_of::<usize>().max(16);
            let layout = Layout::from_size_align(*capacity, align)
                .expect("Invalid layout when dropping BumpAllocator arena");
            unsafe {
                dealloc(ptr.as_ptr(), layout);
            }
        }
    }
}

