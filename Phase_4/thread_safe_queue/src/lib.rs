use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// Error returned by non-blocking push operation when queue is at full capacity.
#[derive(Debug, PartialEq, Eq)]
pub struct TryPushError<T> {
    pub item: T,
}

struct QueueState<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

/// A thread-safe bounded FIFO queue synchronized using standard library
/// `Mutex` and two `Condvar`s (`not_full` and `not_empty`).
pub struct BoundedQueue<T> {
    state: Mutex<QueueState<T>>,
    not_full: Condvar,
    not_empty: Condvar,
}

impl<T> BoundedQueue<T> {
    /// Creates a new `BoundedQueue` with the specified bounded capacity.
    ///
    /// # Panics
    /// Panics if `capacity == 0`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        Self {
            state: Mutex::new(QueueState {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
            }),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
        }
    }

    /// Pushes an item into the queue.
    ///
    /// If the queue is at capacity, the current thread will block until
    /// an item is popped by another thread.
    pub fn push(&self, item: T) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        while state.buffer.len() >= state.capacity {
            state = self.not_full.wait(state).unwrap_or_else(|e| e.into_inner());
        }

        state.buffer.push_back(item);
        self.not_empty.notify_one();
    }

    /// Pops an item from the queue.
    ///
    /// If the queue is empty, the current thread will block until
    /// an item is pushed by another thread.
    pub fn pop(&self) -> T {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        while state.buffer.is_empty() {
            state = self.not_empty.wait(state).unwrap_or_else(|e| e.into_inner());
        }

        let item = state.buffer.pop_front().expect("Queue cannot be empty after wait loop");
        self.not_full.notify_one();
        item
    }

    /// Attempts to push an item without blocking.
    ///
    /// Returns `Err(TryPushError { item })` if the queue is full.
    pub fn try_push(&self, item: T) -> Result<(), TryPushError<T>> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        if state.buffer.len() >= state.capacity {
            return Err(TryPushError { item });
        }

        state.buffer.push_back(item);
        self.not_empty.notify_one();
        Ok(())
    }

    /// Attempts to pop an item without blocking.
    ///
    /// Returns `None` if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        let item = state.buffer.pop_front()?;
        self.not_full.notify_one();
        Some(item)
    }

    /// Returns the number of elements currently in the queue.
    pub fn len(&self) -> usize {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).buffer.len()
    }

    /// Returns `true` if the queue contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the maximum bounded capacity of the queue.
    pub fn capacity(&self) -> usize {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).capacity
    }
}
