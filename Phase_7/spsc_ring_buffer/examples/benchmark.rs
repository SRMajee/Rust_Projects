use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;
use spsc_ring_buffer::{make_spsc, PushError};

const NUM_ITEMS: usize = 5_000_000;
const CAPACITY: usize = 65_536;

fn bench_spsc_ring_buffer() {
    let (mut tx, mut rx) = make_spsc(CAPACITY);

    let start = Instant::now();

    let producer = thread::spawn(move || {
        for i in 0..NUM_ITEMS {
            let mut val = i;
            loop {
                match tx.push(val) {
                    Ok(()) => break,
                    Err(PushError::Full(returned_item)) => {
                        val = returned_item;
                        std::hint::spin_loop();
                    }
                    Err(PushError::Disconnected(_)) => return,
                }
            }
        }
    });

    let consumer = thread::spawn(move || {
        let mut count = 0;
        while count < NUM_ITEMS {
            if let Some(_item) = rx.pop() {
                count += 1;
            } else {
                std::hint::spin_loop();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    let duration = start.elapsed();
    let throughput = (NUM_ITEMS as f64 / duration.as_secs_f64()) / 1_000_000.0;
    println!(
        " [Lock-Free SPSC] Transferred {:>8} items in {:>8.3?} ({:>6.2} M items/sec)",
        NUM_ITEMS, duration, throughput
    );
}

fn bench_std_sync_channel() {
    let (tx, rx) = sync_channel(CAPACITY);

    let start = Instant::now();

    let producer = thread::spawn(move || {
        for i in 0..NUM_ITEMS {
            tx.send(i).unwrap();
        }
    });

    let consumer = thread::spawn(move || {
        let mut count = 0;
        while count < NUM_ITEMS {
            if rx.recv().is_ok() {
                count += 1;
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    let duration = start.elapsed();
    let throughput = (NUM_ITEMS as f64 / duration.as_secs_f64()) / 1_000_000.0;
    println!(
        " [std::sync::mpsc] Transferred {:>8} items in {:>8.3?} ({:>6.2} M items/sec)",
        NUM_ITEMS, duration, throughput
    );
}

fn main() {
    println!("=== Phase 7: SPSC Ring Buffer Benchmark (5,000,000 items) ===");
    bench_spsc_ring_buffer();
    bench_std_sync_channel();
}
