use std::sync::Arc;
use crate::stack::TreiberStack;

/// The multi-producer handle for the MPSC Treiber Stack.
///
/// Can be cloned and shared across multiple threads.
pub struct Producer<T> {
    pub(crate) stack: Arc<TreiberStack<T>>,
}

impl<T> Producer<T> {
    pub(crate) fn new(stack: Arc<TreiberStack<T>>) -> Self {
        Self { stack }
    }

    /// Pushes a value onto the stack using an atomic CAS loop.
    pub fn push(&self, value: T) {
        self.stack.push(value);
    }

    /// Checks if the stack is currently empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl<T> Clone for Producer<T> {
    fn clone(&self) -> Self {
        Self {
            stack: Arc::clone(&self.stack),
        }
    }
}
