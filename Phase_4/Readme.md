# Phase 4: Concurrency & Multithreading

This phase covers multi-threaded programming in Rust, synchronization primitives, shared-state concurrency, and thread pool architectures.

## Projects in this Phase

1. **Thread-Safe In-Memory Queue**
   - **Focus**: `Mutex`, `RwLock`, `Condvar`, condition-based notifications, thread coordination, and safe shared-state access across multiple threads.

2. **Bare-Metal Thread Pool**
   - **Focus**: Worker threads, channel-based job dispatching (`mpsc` or crossbeam channels), graceful shutdown with poison pills, and task scheduling.
