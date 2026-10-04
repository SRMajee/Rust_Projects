//! # Custom Smart Pointers (`MyBox` and `MyRc`)
//!
//! A zero-dependency, educational and foundational implementation of custom smart pointers
//! built directly upon `std::alloc::{alloc, dealloc}`, raw pointers, and Rust's RAII model.
//!
//! ## Implemented Types
//! - [`MyBox`]: Uniquely owned, heap-allocated smart pointer analogous to `std::boxed::Box`.
//! - [`MyRc`]: Non-thread-safe reference-counting smart pointer analogous to `std::rc::Rc`.

pub mod my_box;
pub mod my_rc;

pub use my_box::MyBox;
pub use my_rc::MyRc;

#[cfg(test)]
mod tests;
