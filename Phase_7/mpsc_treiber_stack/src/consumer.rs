use std::sync::Arc;
use crate::stack::{Drain, TreiberStack};

/// The exclusive single-consumer handle for the MPSC Treiber Stack.
///
/// Cannot be cloned. Mutation via `&mut self` statically prevents concurrent pops.
pub struct Consumer<T> {
    pub(crate) stack: Arc<TreiberStack<T>>,
}

impl<T> Consumer<T> {
    pub(crate) fn new(stack: Arc<TreiberStack<T>>) -> Self {
        Self { stack }
    }

    /// Pops a value from the top of the stack.
    ///
    /// Returns `None` if the stack is currently empty.
    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    /// Detaches all currently pending elements in the stack in O(1) time.
    ///
    /// Returns an iterator yielding items from newest to oldest.
    pub fn drain(&mut self) -> Drain<T> {
        self.stack.drain()
    }

    /// Checks if the stack is currently empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}
