use std::ops::{Deref, DerefMut};

/// Cache line size for most modern x86_64 and ARM processors (64 bytes).
pub const CACHE_LINE_SIZE: usize = 64;

/// Align and pad a type `T` to prevent false sharing across CPU cores.
///
/// When multiple threads write or read independent data stored in the same cache line,
/// the CPU cache coherence protocol (e.g., MESI) invalidates the cache line on all other
/// cores, causing significant latency degradation ("false sharing").
///
/// Wrapping atomic indices in `CachePadded` forces the compiler and allocator to align
/// each field to a separate 64-byte boundary.
#[repr(align(64))]
#[derive(Debug)]
pub struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    /// Wrap a value inside `CachePadded`.
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Extract inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Default> Default for CachePadded<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_cache_padded_alignment() {
        assert!(std::mem::align_of::<CachePadded<AtomicUsize>>() >= 64);
        assert!(std::mem::size_of::<CachePadded<AtomicUsize>>() >= 64);
    }
}
