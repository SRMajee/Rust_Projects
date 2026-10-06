//! Custom Bump Allocator (Arena Allocator)
//!
//! A high-performance, thread-safe memory allocator that carves out chunks
//! sequentially from a fixed-size buffer. Supports both typed arena allocation
//! and Rust's `GlobalAlloc` interface.

pub mod bump;
pub use bump::BumpAllocator;

#[cfg(test)]
mod tests;
