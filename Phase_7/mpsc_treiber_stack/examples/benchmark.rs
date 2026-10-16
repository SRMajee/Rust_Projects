use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;
use mpsc_treiber_stack::make_mpsc_stack;

const NUM_ITEMS: usize = 2_000_000;
const NUM_PRODUCERS: usize = 4;
const ITEMS_PER_PRODUCER: usize = NUM_ITEMS / NUM_PRODUCERS;

fn bench_mpsc_treiber_stack() {
    let (tx, mut rx) = make_mpsc_stack();
    let start = Instant::now();

    let mut producer_handles = Vec::with_capacity(NUM_PRODUCERS);
    for p in 0..NUM_PRODUCERS {
        let tx_clone = tx.clone();
        producer_handles.push(thread::spawn(move || {
            let offset = p * ITEMS_PER_PRODUCER;
            for i in 0..ITEMS_PER_PRODUCER {
                tx_clone.push(offset + i);
            }
        }));
    }

    let consumer_handle = thread::spawn(move || {
        let mut count = 0;
        while count < NUM_ITEMS {
            if let Some(_item) = rx.pop() {
                count += 1;
            } else {
                std::hint::spin_loop();
            }
        }
    });

    for h in producer_handles {
        h.join().unwrap();
    }
    consumer_handle.join().unwrap();

    let duration = start.elapsed();
    let throughput = (NUM_ITEMS as f64 / duration.as_secs_f64()) / 1_000_000.0;
    println!(
        " [Lock-Free MPSC Treiber] Transferred {:>8} items in {:>8.3?} ({:>6.2} M items/sec)",
        NUM_ITEMS, duration, throughput
    );
}

fn bench_std_mpsc_channel() {
    let (tx, rx) = channel();
    let start = Instant::now();

    let mut producer_handles = Vec::with_capacity(NUM_PRODUCERS);
    for p in 0..NUM_PRODUCERS {
        let tx_clone = tx.clone();
        producer_handles.push(thread::spawn(move || {
            let offset = p * ITEMS_PER_PRODUCER;
            for i in 0..ITEMS_PER_PRODUCER {
                tx_clone.send(offset + i).unwrap();
            }
        }));
    }

    let consumer_handle = thread::spawn(move || {
        let mut count = 0;
        while count < NUM_ITEMS {
            if rx.recv().is_ok() {
                count += 1;
            }
        }
    });

    for h in producer_handles {
        h.join().unwrap();
    }
    consumer_handle.join().unwrap();

    let duration = start.elapsed();
    let throughput = (NUM_ITEMS as f64 / duration.as_secs_f64()) / 1_000_000.0;
    println!(
        " [Standard std::sync::mpsc] Transferred {:>8} items in {:>8.3?} ({:>6.2} M items/sec)",
        NUM_ITEMS, duration, throughput
    );
}

fn main() {
    println!("============================================================");
    println!(" Running Benchmark: {} items across {} producer threads", NUM_ITEMS, NUM_PRODUCERS);
    println!("============================================================");

    bench_mpsc_treiber_stack();
    bench_std_mpsc_channel();

    println!("============================================================");
}
