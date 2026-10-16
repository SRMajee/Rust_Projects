pub mod cache_pad;
pub mod buffer;
pub mod producer;
pub mod consumer;

use std::sync::Arc;
use buffer::RingBuffer;

pub use cache_pad::CachePadded;
pub use producer::{Producer, PushError};
pub use consumer::{Consumer, PopResult};

/// Creates an ultra-low latency, lock-free Single-Producer Single-Consumer (SPSC) ring buffer.
///
/// The buffer capacity is automatically rounded up to the nearest power of two.
/// Returns a `(Producer<T>, Consumer<T>)` pair with separate ownership.
///
/// # Example
/// ```rust
/// use spsc_ring_buffer::make_spsc;
/// use std::thread;
///
/// let (mut tx, mut rx) = make_spsc(1024);
///
/// let handle = thread::spawn(move || {
///     for i in 0..10_000 {
///         while let Err(spsc_ring_buffer::PushError::Full(val)) = tx.push(i) {
///             std::hint::spin_loop();
///         }
///     }
/// });
///
/// let mut count = 0;
/// while count < 10_000 {
///     if let Some(_val) = rx.pop() {
///         count += 1;
///     } else {
///         std::hint::spin_loop();
///     }
/// }
///
/// handle.join().unwrap();
/// assert_eq!(count, 10_000);
/// ```
pub fn make_spsc<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    let shared = Arc::new(RingBuffer::new(capacity));
    let producer = Producer::new(shared.clone());
    let consumer = Consumer::new(shared);
    (producer, consumer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_basic_push_pop() {
        let (mut tx, mut rx) = make_spsc(4);

        assert_eq!(rx.pop(), None);
        assert_eq!(tx.push(10), Ok(()));
        assert_eq!(tx.push(20), Ok(()));
        assert_eq!(rx.pop(), Some(10));
        assert_eq!(rx.pop(), Some(20));
        assert_eq!(rx.pop(), None);
    }

    #[test]
    fn test_capacity_overflow() {
        let (mut tx, mut rx) = make_spsc(2); // capacity will be 2
        assert_eq!(tx.capacity(), 2);

        assert_eq!(tx.push(1), Ok(()));
        assert_eq!(tx.push(2), Ok(()));
        assert_eq!(tx.push(3), Err(PushError::Full(3)));

        assert_eq!(rx.pop(), Some(1));
        assert_eq!(tx.push(3), Ok(()));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), Some(3));
        assert_eq!(rx.pop(), None);
    }

    #[test]
    fn test_producer_disconnect() {
        let (tx, mut rx) = make_spsc::<i32>(4);
        drop(tx);
        assert_eq!(rx.try_pop(), PopResult::Disconnected);
    }

    #[test]
    fn test_consumer_disconnect() {
        let (mut tx, rx) = make_spsc(4);
        drop(rx);
        assert_eq!(tx.push(42), Err(PushError::Disconnected(42)));
    }

    // Verify unconsumed elements get dropped properly when ring buffer drops
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct DropDetector(#[allow(dead_code)] usize);
    impl Drop for DropDetector {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_drop_leak_safety() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        {
            let (mut tx, _rx) = make_spsc(8);
            for i in 0..5 {
                tx.push(DropDetector(i)).unwrap();
            }
        } // tx and rx dropped here; unconsumed items must be dropped
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_multithreaded_stress() {
        let (mut tx, mut rx) = make_spsc(1024);
        const TOTAL_ITEMS: usize = 500_000;

        let producer_handle = thread::spawn(move || {
            for i in 0..TOTAL_ITEMS {
                let mut item = i;
                loop {
                    match tx.push(item) {
                        Ok(()) => break,
                        Err(PushError::Full(returned_item)) => {
                            item = returned_item;
                            std::hint::spin_loop();
                        }
                        Err(PushError::Disconnected(_)) => panic!("Consumer unexpectedly dropped"),
                    }
                }
            }
        });

        let mut received = 0;
        while received < TOTAL_ITEMS {
            if let Some(val) = rx.pop() {
                assert_eq!(val, received);
                received += 1;
            } else {
                std::hint::spin_loop();
            }
        }

        producer_handle.join().unwrap();
        assert_eq!(received, TOTAL_ITEMS);
    }
}
