pub mod stack;
pub mod producer;
pub mod consumer;

use std::sync::Arc;

pub use stack::{Drain, TreiberStack};
pub use producer::Producer;
pub use consumer::Consumer;

/// Creates an atomic, lock-free Multi-Producer Single-Consumer (MPSC) Treiber Stack.
///
/// Returns a `(Producer<T>, Consumer<T>)` pair.
/// The `Producer` can be cloned across multiple threads, while `Consumer`
/// retains exclusive single-consumer popping rights.
///
/// # Example
/// ```rust
/// use mpsc_treiber_stack::make_mpsc_stack;
/// use std::thread;
///
/// let (tx, mut rx) = make_mpsc_stack();
///
/// let tx2 = tx.clone();
/// let h1 = thread::spawn(move || {
///     tx.push(1);
///     tx.push(2);
/// });
/// let h2 = thread::spawn(move || {
///     tx2.push(3);
///     tx2.push(4);
/// });
///
/// h1.join().unwrap();
/// h2.join().unwrap();
///
/// let mut items = Vec::new();
/// while let Some(val) = rx.pop() {
///     items.push(val);
/// }
/// assert_eq!(items.len(), 4);
/// ```
pub fn make_mpsc_stack<T>() -> (Producer<T>, Consumer<T>) {
    let stack = Arc::new(TreiberStack::new());
    let producer = Producer::new(Arc::clone(&stack));
    let consumer = Consumer::new(stack);
    (producer, consumer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_basic_lifo() {
        let (tx, mut rx) = make_mpsc_stack();
        assert!(rx.is_empty());
        assert_eq!(rx.pop(), None);

        tx.push(1);
        tx.push(2);
        tx.push(3);

        assert!(!rx.is_empty());
        assert_eq!(rx.pop(), Some(3));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), Some(1));
        assert_eq!(rx.pop(), None);
        assert!(rx.is_empty());
    }

    #[test]
    fn test_drain() {
        let (tx, mut rx) = make_mpsc_stack();
        for i in 1..=5 {
            tx.push(i);
        }

        let collected: Vec<_> = rx.drain().collect();
        assert_eq!(collected, vec![5, 4, 3, 2, 1]);
        assert!(rx.is_empty());
        assert_eq!(rx.pop(), None);
    }

    #[test]
    fn test_drop_cleanup() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropDetector;
        impl Drop for DropDetector {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let (tx, _rx) = make_mpsc_stack();
            for _ in 0..100 {
                tx.push(DropDetector);
            }
            // `tx` and `_rx` go out of scope here; all 100 items must be dropped
        }

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_multi_producer_single_consumer() {
        let (tx, mut rx) = make_mpsc_stack();
        let num_threads = 4;
        let items_per_thread = if cfg!(miri) { 50 } else { 2_500 };
        let total_items = num_threads * items_per_thread;

        let mut handles = Vec::new();
        for t in 0..num_threads {
            let tx_clone = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..items_per_thread {
                    tx_clone.push((t * items_per_thread) + i);
                }
            }));
        }

        let consumer_handle = thread::spawn(move || {
            let mut received = Vec::with_capacity(total_items);
            while received.len() < total_items {
                if let Some(val) = rx.pop() {
                    received.push(val);
                } else {
                    std::hint::spin_loop();
                }
            }
            received
        });

        for h in handles {
            h.join().unwrap();
        }

        let received = consumer_handle.join().unwrap();
        assert_eq!(received.len(), total_items);

        let unique: HashSet<_> = received.into_iter().collect();
        assert_eq!(unique.len(), total_items);
    }
}
