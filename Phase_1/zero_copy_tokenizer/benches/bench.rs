use std::hint::black_box;
use std::time::Instant;
use zero_copy_tokenizer::{Tokenizer, UnsafeFastTokenizer};

fn main() {
    println!("============================================================");
    println!("     BENCHMARK: Allocating vs Safe vs Unsafe Tokenizer      ");
    println!("============================================================");

    // Create a realistic sample log & CSV dataset
    let sample_csv_row = "1001,\"Doe, Jane A.\",Senior Cloud Architect,San Francisco,Engineering,Active,185000\n";
    let rows_count = 5_000_000;
    let dataset: String = sample_csv_row.repeat(rows_count); 
    let iterations = 5;

    println!(
        "Dataset size: {:.2} MB ({} rows, ~{} fields)",
        dataset.len() as f64 / 1_048_576.0,
        rows_count,
        rows_count * 7
    );
    println!("Running {} iterations per test...\n", iterations);

    // ---------------------------------------------------------
    // 1. Allocating Approach: clones each token into heap String
    // ---------------------------------------------------------
    let mut total_alloc_duration = std::time::Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let mut total_tokens = 0usize;

        for line in dataset.lines() {
            let tokens: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            total_tokens += tokens.len();
            black_box(&tokens);
        }

        total_alloc_duration += start.elapsed();
        black_box(total_tokens);
    }
    let avg_alloc = total_alloc_duration / iterations;

    // ---------------------------------------------------------
    // 2. Stdlib Split Approach: allocates Vec<&str> buffer per line
    // ---------------------------------------------------------
    let mut total_vec_slice_duration = std::time::Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let mut total_tokens = 0usize;

        for line in dataset.lines() {
            let tokens: Vec<&str> = line.split(',').collect();
            total_tokens += tokens.len();
            black_box(&tokens);
        }

        total_vec_slice_duration += start.elapsed();
        black_box(total_tokens);
    }
    let avg_vec_slice = total_vec_slice_duration / iterations;

    // ---------------------------------------------------------
    // 3. Safe Zero-Copy Tokenizer (Streaming, 0 heap allocations)
    // ---------------------------------------------------------
    let mut total_zero_copy_duration = std::time::Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let mut total_tokens = 0usize;

        for line in dataset.lines() {
            let mut tokenizer = Tokenizer::new(line, ',');
            while let Some(tok) = tokenizer.next() {
                total_tokens += 1;
                black_box(tok);
            }
        }

        total_zero_copy_duration += start.elapsed();
        black_box(total_tokens);
    }
    let avg_zero_copy = total_zero_copy_duration / iterations;

    // ---------------------------------------------------------
    // 4. Unsafe Fast Tokenizer (Raw Byte Pointers, 0 bounds checks)
    // ---------------------------------------------------------
    let mut total_unsafe_fast_duration = std::time::Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let mut total_tokens = 0usize;

        for line in dataset.lines() {
            let mut tokenizer = UnsafeFastTokenizer::new(line, b',');
            while let Some(tok) = tokenizer.next() {
                total_tokens += 1;
                black_box(tok);
            }
        }

        total_unsafe_fast_duration += start.elapsed();
        black_box(total_tokens);
    }
    let avg_unsafe_fast = total_unsafe_fast_duration / iterations;

    // ---------------------------------------------------------
    // Results Summary
    // ---------------------------------------------------------
    println!("----------------------------------------------------------------------");
    println!("RESULTS (Average execution time):");
    println!("----------------------------------------------------------------------");
    println!(
        "1. Allocating (Vec<String>)            : {:>8.2?}  (Heap allocates every field)",
        avg_alloc
    );
    println!(
        "2. Slice Buffer (Vec<&str>)            : {:>8.2?}  (Heap allocates token vector)",
        avg_vec_slice
    );
    println!(
        "3. Safe Zero-Copy (Tokenizer)          : {:>8.2?}  (Safe, 0 heap allocations)",
        avg_zero_copy
    );
    println!(
        "4. Unsafe Fast (UnsafeFastTokenizer)   : {:>8.2?}  (Raw pointers, SIMD/unrolled)",
        avg_unsafe_fast
    );
    println!("----------------------------------------------------------------------");
    let speedup_vs_alloc = avg_alloc.as_secs_f64() / avg_unsafe_fast.as_secs_f64();
    let speedup_vs_vec = avg_vec_slice.as_secs_f64() / avg_unsafe_fast.as_secs_f64();
    let speedup_vs_safe = avg_zero_copy.as_secs_f64() / avg_unsafe_fast.as_secs_f64();

    println!(">>> UnsafeFastTokenizer is {:.2}x FASTER than Safe Tokenizer!", speedup_vs_safe);
    println!(">>> UnsafeFastTokenizer is {:.2}x FASTER than collecting Vec<&str>!", speedup_vs_vec);
    println!(">>> UnsafeFastTokenizer is {:.2}x FASTER than allocating String!", speedup_vs_alloc);
    println!("======================================================================");
}
