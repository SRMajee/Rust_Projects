use std::sync::atomic::Ordering;
use std::sync::Arc;
use crate::buffer::RingBuffer;

/// Result of attempting to pop an element from the SPSC Ring Buffer.
#[derive(Debug, PartialEq, Eq)]
pub enum PopResult<T> {
    /// Item successfully read.
    Ok(T),
    /// Buffer is currently empty, but producer is still alive.
    Empty,
    /// Buffer is empty AND producer has dropped.
    Disconnected,
}

/// Consumer handle of the SPSC Ring Buffer.
///
/// Guaranteed to be single-consumer: cannot be cloned.
pub struct Consumer<T> {
    pub(crate) shared: Arc<RingBuffer<T>>,
    /// Local non-atomic copy of tail index. Only modified by this consumer.
    pub(crate) local_tail: usize,
    /// Cached copy of the producer's head.
    /// Greatly reduces inter-core atomic reads when multiple items are available.
    pub(crate) cached_head: usize,
}

// Safety: Consumer owns the single-reader privilege.
unsafe impl<T: Send> Send for Consumer<T> {}

impl<T> Consumer<T> {
    pub(crate) fn new(shared: Arc<RingBuffer<T>>) -> Self {
        Self {
            shared,
            local_tail: 0,
            cached_head: 0,
        }
    }

    /// Attempts to pop an element from the ring buffer without blocking.
    ///
    /// # Memory Ordering:
    /// 1. Verifies if items are available using local `cached_head`.
    /// 2. If locally empty, loads `shared.head` with `Ordering::Acquire`.
    /// 3. Reads element from `UnsafeCell<MaybeUninit<T>>` via `ptr::read`.
    /// 4. Stores updated tail with `Ordering::Release`, freeing the slot for the producer.
    pub fn pop(&mut self) -> Option<T> {
        match self.try_pop() {
            PopResult::Ok(val) => Some(val),
            PopResult::Empty | PopResult::Disconnected => None,
        }
    }

    /// Attempts to pop with explicit awareness of producer disconnection.
    pub fn try_pop(&mut self) -> PopResult<T> {
        // Check if buffer is empty using local cached_head first (zero atomic contention)
        if self.local_tail == self.cached_head {
            // Reload the real head with Acquire ordering to synchronize data written by producer
            self.cached_head = self.shared.head.load(Ordering::Acquire);

            // If still empty, check if producer is disconnected
            if self.local_tail == self.cached_head {
                if !self.is_producer_alive() {
                    return PopResult::Disconnected;
                }
                return PopResult::Empty;
            }
        }

        // Calculate slot index using bitmask
        let slot_idx = self.local_tail & self.shared.mask;

        // Safety:
        // - Single consumer: Only this thread reads and advances tail.
        // - Producer published slot via head with Release ordering, which was acquired above.
        // - Memory at slot_idx contains a valid, initialized instance of T.
        let item = unsafe {
            let slot_ptr = self.shared.buffer[slot_idx].get();
            std::ptr::read(slot_ptr as *const T)
        };

        // Advance local tail and publish to producer
        self.local_tail = self.local_tail.wrapping_add(1);
        self.shared.tail.store(self.local_tail, Ordering::Release);

        PopResult::Ok(item)
    }

    /// Check if the producer is still alive.
    #[inline]
    pub fn is_producer_alive(&self) -> bool {
        self.shared.producer_alive.load(Ordering::Relaxed)
    }

    /// Returns the buffer capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.shared.capacity()
    }
}

impl<T> Drop for Consumer<T> {
    fn drop(&mut self) {
        self.shared.consumer_alive.store(false, Ordering::Release);
    }
}
