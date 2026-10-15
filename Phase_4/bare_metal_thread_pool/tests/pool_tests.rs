use bare_metal_thread_pool::{PoolCreationError, ThreadPool};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_pool_creation_and_size() {
    let pool = ThreadPool::new(4);
    assert_eq!(pool.size(), 4);

    let build_res = ThreadPool::build(0);
    assert_eq!(build_res.err(), Some(PoolCreationError::ZeroThreads));
}

#[test]
#[should_panic(expected = "ThreadPool size must be greater than zero")]
fn test_pool_new_zero_panics() {
    let _ = ThreadPool::new(0);
}

#[test]
fn test_execute_single_job() {
    let pool = ThreadPool::new(2);
    let counter = Arc::new(AtomicUsize::new(0));

    let c = Arc::clone(&counter);
    pool.execute(move || {
        c.fetch_add(10, Ordering::SeqCst);
    });

    drop(pool); // Dropping pool waits for jobs to finish
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_execute_multiple_concurrent_jobs() {
    let num_workers = 4;
    let pool = ThreadPool::new(num_workers);
    let job_count = 100;
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..job_count {
        let c = Arc::clone(&counter);
        pool.execute(move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
    }

    drop(pool);
    assert_eq!(counter.load(Ordering::SeqCst), job_count);
}

#[test]
fn test_threads_execute_in_parallel() {
    let num_workers = 3;
    let pool = ThreadPool::new(num_workers);
    let barrier = Arc::new(Barrier::new(num_workers));

    for _ in 0..num_workers {
        let b = Arc::clone(&barrier);
        pool.execute(move || {
            // If threads run in parallel, all will arrive at the barrier concurrently
            b.wait();
        });
    }

    drop(pool);
}

#[test]
fn test_explicit_shutdown() {
    let mut pool = ThreadPool::new(2);
    let results = Arc::new(Mutex::new(Vec::new()));

    for i in 0..5 {
        let res = Arc::clone(&results);
        pool.execute(move || {
            thread::sleep(Duration::from_millis(5));
            res.lock().unwrap().push(i);
        });
    }

    pool.shutdown();
    assert_eq!(results.lock().unwrap().len(), 5);
}

#[test]
fn test_graceful_shutdown_finishes_queued_tasks() {
    let pool = ThreadPool::new(2);
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..20 {
        let c = Arc::clone(&counter);
        pool.execute(move || {
            thread::sleep(Duration::from_millis(2));
            c.fetch_add(1, Ordering::SeqCst);
        });
    }

    // Drop triggers shutdown & joins all worker threads
    drop(pool);
    assert_eq!(counter.load(Ordering::SeqCst), 20);
}
