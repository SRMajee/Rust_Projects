use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::cache_pad::CachePadded;

/// Internal shared ring buffer representation.
pub(crate) struct RingBuffer<T> {
    /// Contiguous heap buffer of uninitialized slots.
    pub(crate) buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    /// Bitmask for fast power-of-two index wrapping: `index & mask`.
    pub(crate) mask: usize,
    /// Head index: where the producer writes the next element.
    /// Padded to a dedicated 64-byte cache line to prevent false sharing.
    pub(crate) head: CachePadded<AtomicUsize>,
    /// Tail index: where the consumer reads the next element.
    /// Padded to a dedicated 64-byte cache line to prevent false sharing.
    pub(crate) tail: CachePadded<AtomicUsize>,
    /// Disconnection flags to gracefully handle drop of Producer or Consumer.
    pub(crate) producer_alive: AtomicBool,
    pub(crate) consumer_alive: AtomicBool,
}

// Safety: RingBuffer manages concurrent access via atomic head & tail indices.
// UnsafeCell is safe to share across threads when access follows SPSC protocols.
unsafe impl<T: Send> Sync for RingBuffer<T> {}
unsafe impl<T: Send> Send for RingBuffer<T> {}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with capacity rounded up to the nearest power of two.
    pub(crate) fn new(requested_capacity: usize) -> Self {
        let capacity = requested_capacity.max(2).next_power_of_two();
        let mask = capacity - 1;

        // Allocate uninitialized storage for the slots
        let mut buffer_vec = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer_vec.push(UnsafeCell::new(MaybeUninit::uninit()));
        }

        Self {
            buffer: buffer_vec.into_boxed_slice(),
            mask,
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
            producer_alive: AtomicBool::new(true),
            consumer_alive: AtomicBool::new(true),
        }
    }

    /// Total capacity of the ring buffer.
    #[inline(always)]
    pub(crate) fn capacity(&self) -> usize {
        self.mask + 1
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        // Drop any remaining unconsumed elements in the buffer to prevent memory leaks.
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        for index in tail..head {
            let slot_idx = index & self.mask;
            unsafe {
                let slot_ptr = self.buffer[slot_idx].get();
                // Safety: Every slot between tail and head was initialized and not yet popped.
                (*slot_ptr).assume_init_drop();
            }
        }
    }
}
