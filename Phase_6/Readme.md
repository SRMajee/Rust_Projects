# Phase 6: Asynchronous Rust

This phase investigates the internal mechanics of asynchronous programming in Rust, cooperative multitasking, and event loop architecture.

## Projects in this Phase

1. **Hand-Rolled Future & Mini-Executor** (`mini_executor`)
   - **Focus**: `Future` trait implementation, `Poll`, `Context`, `Waker` mechanisms, event scheduling, and building a mini reactor/executor loop from scratch without third-party runtimes.
   - **Implemented Components**:
     - `Task` implementing `std::task::Wake` for self-rescheduling back to the executor channel via `Arc<Task>`.
     - `Executor` & `Spawner` single-threaded event loop driven by `std::sync::mpsc`.
     - `TimerFuture` demonstrating reactor-style event completion and waking.
     - `YieldNow` (`yield_now()`) cooperative multitasking state machine.
     - Integration test suite covering task interleaving, wakeup sequencing, and timer ordering.
