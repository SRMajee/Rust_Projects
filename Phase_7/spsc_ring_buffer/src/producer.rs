use std::sync::atomic::Ordering;
use std::sync::Arc;
use crate::buffer::RingBuffer;

/// Error returned when pushing an element into a full buffer or disconnected consumer.
#[derive(Debug, PartialEq, Eq)]
pub enum PushError<T> {
    /// Buffer is full; contains the unpushed item.
    Full(T),
    /// Consumer has disconnected / dropped; contains the unpushed item.
    Disconnected(T),
}

/// Producer handle of the SPSC Ring Buffer.
///
/// Guaranteed to be single-producer: cannot be cloned.
pub struct Producer<T> {
    pub(crate) shared: Arc<RingBuffer<T>>,
    /// Local non-atomic copy of head index. Only written/modified by this producer.
    pub(crate) local_head: usize,
    /// Cached copy of the consumer's tail.
    /// Greatly reduces atomic reads to the shared tail on the consumer's core.
    pub(crate) cached_tail: usize,
}

// Safety: Producer owns the single-writer privilege.
unsafe impl<T: Send> Send for Producer<T> {}

impl<T> Producer<T> {
    pub(crate) fn new(shared: Arc<RingBuffer<T>>) -> Self {
        Self {
            shared,
            local_head: 0,
            cached_tail: 0,
        }
    }

    /// Attempts to push an element into the ring buffer without blocking.
    ///
    /// # Memory Ordering:
    /// 1. Verifies space available using local `cached_tail`.
    /// 2. If locally full, loads `shared.tail` with `Ordering::Acquire`.
    /// 3. Writes element into `UnsafeCell<MaybeUninit<T>>` via `ptr::write`.
    /// 4. Stores updated head with `Ordering::Release`, making the written payload visible to consumer.
    pub fn push(&mut self, item: T) -> Result<(), PushError<T>> {
        if !self.is_consumer_alive() {
            return Err(PushError::Disconnected(item));
        }

        let capacity = self.shared.capacity();

        // Check if buffer is full using our local cached_tail first (zero atomic contention)
        if self.local_head.wrapping_sub(self.cached_tail) >= capacity {
            // Reload the real tail with Acquire ordering
            self.cached_tail = self.shared.tail.load(Ordering::Acquire);

            // Re-check after refreshing cached tail
            if self.local_head.wrapping_sub(self.cached_tail) >= capacity {
                return Err(PushError::Full(item));
            }
        }

        // Calculate slot index using bitmask
        let slot_idx = self.local_head & self.shared.mask;

        // Safety:
        // - Single producer: Only this thread writes to slot_idx at local_head.
        // - Consumer cannot read this slot because shared.head has not been updated yet.
        // - Buffer space verified above ensures slot is not currently occupied.
        unsafe {
            let slot_ptr = self.shared.buffer[slot_idx].get();
            (*slot_ptr).write(item);
        }

        // Advance local head and publish to consumer
        self.local_head = self.local_head.wrapping_add(1);
        self.shared.head.store(self.local_head, Ordering::Release);

        Ok(())
    }

    /// Check if the consumer is still alive.
    #[inline]
    pub fn is_consumer_alive(&self) -> bool {
        self.shared.consumer_alive.load(Ordering::Relaxed)
    }

    /// Returns the buffer capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.shared.capacity()
    }
}

impl<T> Drop for Producer<T> {
    fn drop(&mut self) {
        self.shared.producer_alive.store(false, Ordering::Release);
    }
}
