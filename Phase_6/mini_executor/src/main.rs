use mini_executor::{yield_now, Executor, TimerFuture};
use std::time::Duration;

fn main() {
    println!("=== Phase 6: Hand-Rolled Future & Mini-Executor Demo ===");

    let (executor, spawner) = Executor::new(100);

    // Task 1: Async delay via custom TimerFuture
    let spawner_clone = spawner.clone();
    spawner.spawn(async move {
        println!("[Task 1] Started. Waiting 50ms on TimerFuture...");
        TimerFuture::new(Duration::from_millis(50)).await;
        println!("[Task 1] Timer completed! Spawning Sub-Task 1B...");

        spawner_clone.spawn(async {
            println!("[Sub-Task 1B] Executing inside spawned subtask!");
        });
    });

    // Task 2: Cooperative multitasking using custom YieldNow
    spawner.spawn(async {
        println!("[Task 2] Step A: Yielding turn cooperatively...");
        yield_now().await;
        println!("[Task 2] Step B: Yielding turn cooperatively again...");
        yield_now().await;
        println!("[Task 2] Step C: Finished!");
    });

    // Task 3: Another concurrent task
    spawner.spawn(async {
        println!("[Task 3] Concurrent task started and done immediately.");
    });

    // Drop original spawner so that when all tasks finish, the channel closes
    // and executor.run() cleanly terminates.
    drop(spawner);

    println!("Starting executor event loop...");
    executor.run();
    println!("Executor event loop finished. All tasks completed!");
}
