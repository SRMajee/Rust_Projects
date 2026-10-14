use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thread_safe_queue::{BoundedQueue, TryPushError};

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_send_sync_traits() {
    assert_send_sync::<BoundedQueue<i32>>();
    assert_send_sync::<BoundedQueue<String>>();
}

#[test]
fn test_single_threaded_fifo() {
    let queue = BoundedQueue::new(3);
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.capacity(), 3);

    queue.push(10);
    queue.push(20);
    queue.push(30);

    assert_eq!(queue.len(), 3);
    assert_eq!(queue.pop(), 10);
    assert_eq!(queue.pop(), 20);
    assert_eq!(queue.pop(), 30);
    assert!(queue.is_empty());
}

#[test]
fn test_try_push_and_try_pop() {
    let queue = BoundedQueue::new(2);

    assert_eq!(queue.try_pop(), None);
    assert_eq!(queue.try_push(1), Ok(()));
    assert_eq!(queue.try_push(2), Ok(()));
    assert_eq!(queue.try_push(3), Err(TryPushError { item: 3 }));

    assert_eq!(queue.try_pop(), Some(1));
    assert_eq!(queue.try_push(3), Ok(()));
    assert_eq!(queue.try_pop(), Some(2));
    assert_eq!(queue.try_pop(), Some(3));
    assert_eq!(queue.try_pop(), None);
}

#[test]
#[should_panic(expected = "Capacity must be greater than zero")]
fn test_zero_capacity_panics() {
    let _queue: BoundedQueue<i32> = BoundedQueue::new(0);
}

#[test]
fn test_bounded_blocking_push() {
    let queue = Arc::new(BoundedQueue::new(2));
    queue.push(1);
    queue.push(2);

    let pushed_flag = Arc::new(AtomicBool::new(false));
    let q_clone = Arc::clone(&queue);
    let flag_clone = Arc::clone(&pushed_flag);

    let handle = thread::spawn(move || {
        // This push must block because queue capacity is 2
        q_clone.push(3);
        flag_clone.store(true, Ordering::SeqCst);
    });

    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        pushed_flag.load(Ordering::SeqCst),
        false,
        "Producer should be blocked because queue is full"
    );

    // Pop an item to free space
    assert_eq!(queue.pop(), 1);

    handle.join().expect("Producer thread should complete");
    assert_eq!(pushed_flag.load(Ordering::SeqCst), true);
    assert_eq!(queue.pop(), 2);
    assert_eq!(queue.pop(), 3);
}

#[test]
fn test_blocking_pop() {
    let queue = Arc::new(BoundedQueue::new(2));
    let q_clone = Arc::clone(&queue);

    let handle = thread::spawn(move || {
        // Will block until main thread pushes
        q_clone.pop()
    });

    thread::sleep(Duration::from_millis(50));
    queue.push(999);

    let popped = handle.join().expect("Consumer thread should complete");
    assert_eq!(popped, 999);
}

#[test]
fn test_mpmc_stress() {
    let queue = Arc::new(BoundedQueue::new(10));
    let num_producers = 4;
    let num_consumers = 4;
    let items_per_producer = 500;
    let total_items = num_producers * items_per_producer;

    let mut producer_handles = Vec::new();
    for p_id in 0..num_producers {
        let q = Arc::clone(&queue);
        producer_handles.push(thread::spawn(move || {
            for i in 0..items_per_producer {
                q.push(p_id * items_per_producer + i);
            }
        }));
    }

    let mut consumer_handles = Vec::new();
    for _ in 0..num_consumers {
        let q = Arc::clone(&queue);
        consumer_handles.push(thread::spawn(move || {
            let mut received = Vec::new();
            for _ in 0..(total_items / num_consumers) {
                received.push(q.pop());
            }
            received
        }));
    }

    for h in producer_handles {
        h.join().expect("Producer finished");
    }

    let mut all_received = Vec::new();
    for h in consumer_handles {
        all_received.extend(h.join().expect("Consumer finished"));
    }

    assert_eq!(all_received.len(), total_items);
    all_received.sort();
    let expected: Vec<usize> = (0..total_items).collect();
    assert_eq!(all_received, expected);
}
