# Phase 7: Lock-Free & High-Frequency Systems

This phase tackles advanced low-latency and lock-free systems programming, atomic instructions, memory orderings, and cache-friendly data structures.

## Projects in this Phase

1. **Single-Producer Single-Consumer (SPSC) Ring Buffer**
   - **Focus**: Lock-free circular ring buffer, atomic head/tail indices, `AtomicUsize`, cache-line padding (`#[repr(align(64))]`) to prevent false sharing, and `Acquire`/`Release` memory ordering semantics.

2. **Multi-Producer Single-Consumer (MPSC) Treiber Stack**
   - **Focus**: Lock-free stack using atomic compare-and-swap (`compare_exchange`), ABA problem prevention considerations, and concurrent non-blocking data exchange.
