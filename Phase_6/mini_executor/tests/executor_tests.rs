use mini_executor::{yield_now, Executor, TimerFuture};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[test]
fn test_single_task_completion() {
    let (executor, spawner) = Executor::new(10);
    let counter = Arc::new(AtomicUsize::new(0));

    let c = Arc::clone(&counter);
    spawner.spawn(async move {
        c.fetch_add(1, Ordering::SeqCst);
    });

    drop(spawner);
    executor.run();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_timer_future_and_wakeup() {
    let (executor, spawner) = Executor::new(10);
    let done = Arc::new(AtomicUsize::new(0));

    let d = Arc::clone(&done);
    let start = Instant::now();
    spawner.spawn(async move {
        TimerFuture::new(Duration::from_millis(30)).await;
        d.store(1, Ordering::SeqCst);
    });

    drop(spawner);
    executor.run();

    assert_eq!(done.load(Ordering::SeqCst), 1);
    assert!(start.elapsed() >= Duration::from_millis(25));
}

#[test]
fn test_cooperative_interleaving() {
    let (executor, spawner) = Executor::new(10);
    let log = Arc::new(Mutex::new(Vec::new()));

    let log_1 = Arc::clone(&log);
    spawner.spawn(async move {
        log_1.lock().unwrap().push("T1_1");
        yield_now().await;
        log_1.lock().unwrap().push("T1_2");
    });

    let log_2 = Arc::clone(&log);
    spawner.spawn(async move {
        log_2.lock().unwrap().push("T2_1");
        yield_now().await;
        log_2.lock().unwrap().push("T2_2");
    });

    drop(spawner);
    executor.run();

    let recorded = log.lock().unwrap().clone();
    // Interleaving verification: Both tasks should start before either finishes their second step
    assert_eq!(recorded.len(), 4);
    assert_eq!(recorded[0], "T1_1");
    assert_eq!(recorded[1], "T2_1");
    assert_eq!(recorded[2], "T1_2");
    assert_eq!(recorded[3], "T2_2");
}

#[test]
fn test_multiple_timers_ordering() {
    let (executor, spawner) = Executor::new(10);
    let order = Arc::new(Mutex::new(Vec::new()));

    let o1 = Arc::clone(&order);
    spawner.spawn(async move {
        // Longer timer
        TimerFuture::new(Duration::from_millis(50)).await;
        o1.lock().unwrap().push("50ms");
    });

    let o2 = Arc::clone(&order);
    spawner.spawn(async move {
        // Shorter timer
        TimerFuture::new(Duration::from_millis(15)).await;
        o2.lock().unwrap().push("15ms");
    });

    drop(spawner);
    executor.run();

    let result = order.lock().unwrap().clone();
    assert_eq!(result, vec!["15ms", "50ms"]);
}
