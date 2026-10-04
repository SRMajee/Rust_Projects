# Zero-Copy String Tokenizer / Scanner

A high-performance, allocation-free string tokenizer and scanner implemented in Rust, built as part of **Phase 1: Foundations & Memory Management**.

---

## 📖 Overview

In systems programming, network proxies, log parsers, and data pipelines, throughput is often bottle-necked by dynamic memory allocations (`malloc` / `String::clone` / `Vec`).

This project demonstrates **Zero-Copy Parsing**:

- Slices the input string directly in place into borrowed references (`&'a str`).
- Allocates **0 bytes of heap memory** during iteration.
- Enforces strict compile-time safety and lifetime invariants via the Rust borrow checker.

---

## 🎯 Project Objectives & Constraints

- **Language & Edition**: Rust (2024 edition)
- **External Dependencies**: Zero (`std` only)
- **Target Footprint**: Clean, idiomatic, compact implementation (~40–100 lines)
- **Memory Guarantee**: No heap allocation during tokenization

---

## 🧩 Step-by-Step Implementation

The project was constructed following a structured 4-step architectural approach:

### Step 1: Modeling Borrowed Tokens & Lifetimes (`Token<'a>`)

Rather than copying substrings into new `String` instances, we define a lightweight struct [`Token<'a>`](file:///c:/DRIVE%20D/Rust_Projects/Phase_1/zero_copy_tokenizer/src/lib.rs#L8-L41) that holds:

- `value: &'a str`: Direct borrow pointing into the source buffer.
- `start_offset: usize` & `end_offset: usize`: Exact byte positions within the parent input.

```rust
pub struct Token<'a> {
    pub value: &'a str,
    pub start_offset: usize,
    pub end_offset: usize,
}
```

### Step 2: Designing Cursor State Without Heap Allocations

The [`Tokenizer<'a>`](file:///c:/DRIVE%20D/Rust_Projects/Phase_1/zero_copy_tokenizer/src/lib.rs#L48-L135) struct maintains parser state across successive steps using only stack primitives:

- `remainder: &'a str`: The unconsumed tail of the input slice.
- `delimiter: char`: Delimiter to split by.
- `cursor: usize`: Current byte position for reporting token offsets.
- `handle_quotes: bool`: Flag to support quoted strings containing delimiters.
- `finished: bool`: End-of-stream flag.

### Step 3: Implementing the `Iterator` Trait

By implementing the standard `Iterator` trait:

```rust
impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().map(|t| t.value)
    }
}
```

This unlocks full compatibility with Rust's iterator combinators (`.map()`, `.filter()`, `.take()`, `for` loops, etc.) with zero runtime overhead.

### Step 4: Verification & Test Coverage

Comprehensive unit and documentation tests cover:

1. **Space-delimited network/log format**: Parsing standard server log lines.
2. **CSV records with consecutive delimiters**: Properly retaining empty fields `""`.
3. **Quoted CSV values**: Slicing strings containing inner delimiters without false splits.
4. **Byte boundary verification**: Exact token byte range tracking.
5. **Edge cases**: Empty inputs, single token inputs, and trailing tokens.

---

## 🚀 Usage Examples

### 1. Basic Tokenization (Log Parsing)

```rust
use zero_copy_tokenizer::Tokenizer;

let log_line = "2026-10-03 INFO [worker-1] Task finished";
let mut tokenizer = Tokenizer::new(log_line, ' ');

assert_eq!(tokenizer.next(), Some("2026-10-03"));
assert_eq!(tokenizer.next(), Some("INFO"));
assert_eq!(tokenizer.next(), Some("[worker-1]"));
```

### 2. Quoted CSV Parsing

```rust
use zero_copy_tokenizer::Tokenizer;

let csv_row = "101,\"Doe, Jane\",Engineer,95000";
let fields: Vec<&str> = Tokenizer::new(csv_row, ',')
    .with_quotes()
    .collect();

assert_eq!(fields, vec!["101", "Doe, Jane", "Engineer", "95000"]);
```

### 3. Accessing Token Byte Offsets

```rust
use zero_copy_tokenizer::Tokenizer;

let input = "GET /index.html HTTP/1.1";
let mut tokenizer = Tokenizer::new(input, ' ');

let token = tokenizer.next_token().unwrap();
assert_eq!(token.value, "GET");
assert_eq!(token.start_offset, 0);
assert_eq!(token.end_offset, 4);
```

---

## 🧪 Running the Tests

To verify the test suite and documentation examples:

```powershell
cargo test
```

All unit tests and doctests execute and pass with zero warnings.

---

## ⚡ Performance Benchmarks

A real-world benchmark suite is included in [`benches/bench.rs`](file:///c:/DRIVE%20D/Rust_Projects/Phase_1/zero_copy_tokenizer/benches/bench.rs). It benchmarks parsing **500,000 CSV lines (3,500,000 fields, ~40 MB dataset)** comparing:

1. **Allocating Approach (`Vec<String>`)**: Naive splitting where every token allocates a heap `String`.
2. **Slice Buffer (`Vec<&str>`)**: Using `.split(',')` and accumulating slices into a heap `Vec`.
3. **Safe Zero-Copy (`Tokenizer`)**: Yields borrowed tokens with 0 heap allocations, safe bounds checks, and UTF-8 validation.
4. **Unsafe Fast (`UnsafeFastTokenizer`)**: Direct raw-pointer scanning with 0 bounds checks, auto-vectorization, and unchecked UTF-8 slice emission.

### Benchmark Results (`cargo bench`)

| Strategy | Memory Overhead | Execution Time | Speedup vs Allocating | Speedup vs Safe Tokenizer |
| :--- | :--- | :--- | :--- | :--- |
| **Allocating (`Vec<String>`)** | 3.5M+ heap allocations | **304.50 ms** | Baseline (1.00x) | — |
| **Slice Buffer (`Vec<&str>`)** | 500K heap `Vec` allocations | **108.96 ms** | ~2.80x faster | — |
| **Safe Zero-Copy (`Tokenizer`)** | **0 heap allocations** | **63.82 ms** | ~4.77x faster | Baseline |
| **Unsafe Fast (`UnsafeFastTokenizer`)** | **0 heap allocations** | **28.29 ms** | **~10.76x faster** | **~2.26x faster** |

Run the benchmarks yourself with:

```powershell
cargo bench
```
