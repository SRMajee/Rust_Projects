use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

/// An atomic, lock-free Multi-Producer Single-Consumer (MPSC) Treiber Stack.
pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    /// Creates a new empty `TreiberStack`.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Pushes a value onto the top of the stack using an atomic CAS loop.
    ///
    /// This operation is lock-free and thread-safe for multiple concurrent producers.
    pub fn push(&self, value: T) {
        let new_node = Box::into_raw(Box::new(Node {
            value,
            next: ptr::null_mut(),
        }));

        let mut current = self.head.load(Ordering::Relaxed);
        loop {
            // SAFETY: `new_node` is a uniquely owned pointer just allocated from `Box::into_raw`.
            unsafe {
                (*new_node).next = current;
            }

            // Publish `new_node` as the new head.
            // Release ordering ensures that the contents of `new_node` are visible
            // to any thread reading `head` with Acquire ordering.
            match self.head.compare_exchange_weak(
                current,
                new_node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    /// Pops a value from the top of the stack.
    ///
    /// # Safety / Concurrency
    /// In an MPSC model, this method must only be called by a single consumer at a time.
    /// The single-consumer invariant guarantees that no concurrent pop occurs,
    /// eliminating the classic Treiber stack ABA use-after-free hazard.
    pub fn pop(&self) -> Option<T> {
        let mut current = self.head.load(Ordering::Acquire);
        loop {
            if current.is_null() {
                return None;
            }

            // SAFETY: `current` is non-null and was obtained from `head`.
            // Because only this consumer thread deallocates nodes, and concurrent
            // producers only prepend new nodes, `current` remains valid memory.
            let next = unsafe { (*current).next };

            match self.head.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // SAFETY: We atomically unlinked `current` from the stack head.
                    // As the sole consumer, we now hold unique ownership and can reclaim it.
                    let boxed = unsafe { Box::from_raw(current) };
                    return Some(boxed.value);
                }
                Err(actual) => current = actual,
            }
        }
    }

    /// Detaches all currently queued elements from the stack in O(1) time.
    ///
    /// Returns an iterator yielding items from newest to oldest.
    pub fn drain(&self) -> Drain<T> {
        // Atomically replace head with null, detaching the entire list.
        let head = self.head.swap(ptr::null_mut(), Ordering::AcqRel);
        Drain { head }
    }

    /// Checks if the stack is currently empty.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }
}

impl<T> Default for TreiberStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        // Iteratively deallocate remaining nodes to prevent call-stack overflow.
        let mut curr = self.head.swap(ptr::null_mut(), Ordering::Relaxed);
        while !curr.is_null() {
            // SAFETY: In drop, we have exclusive ownership of the stack.
            let next = unsafe { (*curr).next };
            drop(unsafe { Box::from_raw(curr) });
            curr = next;
        }
    }
}

/// An iterator that drains nodes from a detached Treiber Stack segment.
pub struct Drain<T> {
    head: *mut Node<T>,
}

unsafe impl<T: Send> Send for Drain<T> {}

impl<T> Iterator for Drain<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            None
        } else {
            let curr = self.head;
            // SAFETY: Exclusive ownership of the unlinked chain.
            let next = unsafe { (*curr).next };
            let boxed = unsafe { Box::from_raw(curr) };
            self.head = next;
            Some(boxed.value)
        }
    }
}

impl<T> Drop for Drain<T> {
    fn drop(&mut self) {
        for _ in self.by_ref() {}
    }
}
